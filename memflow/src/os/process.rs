//! Describes process context

use super::{ModuleAddressInfo, ModuleInfo, ModuleInfoCallback};
use crate::prelude::v1::{Result, *};
use std::prelude::v1::*;

/// Type meant for process IDs
///
/// If there is a case where PID can be over 32-bit limit, or negative, please open an issue, we
/// would love to see that.
pub type PID = u32;

/// Provides all actions on processes
///
/// This trait provides a lot of typical functionality for processes, such as memory access, module lists, and basic information.
///
/// Future expansions could include threads, keyboard input, and more.
pub trait Process: Send {
    type VirtualMemoryType: VirtualMemory;
    //type VirtualTranslateType: VirtualTranslate;

    /// Retrieves virtual memory object for the process
    fn virt_mem(&mut self) -> &mut Self::VirtualMemoryType;

    /// Retrieves virtual address translator for the process (if applicable)
    //fn vat(&mut self) -> Option<&mut Self::VirtualTranslateType>;

    /// Walks the process' module list and calls the provided callback for each module structure
    /// address
    ///
    /// # Arguments
    /// * `target_arch` - sets which architecture to retrieve the modules for (if emulated). Choose
    /// between `Some(ProcessInfo::sys_arch())`, and `Some(ProcessInfo::proc_arch())`. `None` for all.
    /// * `callback` - where to pass each matching module to. This is an opaque callback.
    fn module_address_list_callback(
        &mut self,
        target_arch: Option<&ArchitectureIdent>,
        callback: ModuleAddressCallback,
    ) -> Result<()>;

    /// Walks the process' module list and calls the provided callback for each module
    ///
    /// # Arguments
    /// * `target_arch` - sets which architecture to retrieve the modules for (if emulated). Choose
    /// between `Some(ProcessInfo::sys_arch())`, and `Some(ProcessInfo::proc_arch())`. `None` for all.
    /// * `callback` - where to pass each matching module to. This is an opaque callback.
    fn module_list_callback(
        &mut self,
        target_arch: Option<&ArchitectureIdent>,
        mut callback: ModuleInfoCallback,
    ) -> Result<()> {
        // This is safe, because control will flow back to the callback.
        let sptr = self as *mut Self;
        let inner_callback = &mut |ModuleAddressInfo { address, arch }| match unsafe { &mut *sptr }
            .module_by_address(address, arch)
        {
            Ok(info) => callback.call(info),
            Err(e) => {
                log::trace!("Error loading module {:x} {:?}", address, e);
                false
            }
        };
        unsafe { sptr.as_mut().unwrap() }
            .module_address_list_callback(target_arch, inner_callback.into())
    }

    /// Retreives a module by its structure address and architecture
    ///
    /// # Arguments
    /// * `address` - address where module's information resides in
    /// * `architecture` - architecture of the module. Should be either `ProcessInfo::proc_arch`, or `ProcessInfo::sys_arch`.
    fn module_by_address(
        &mut self,
        address: Address,
        architecture: ArchitectureIdent,
    ) -> Result<ModuleInfo>;

    /// Finds a process module by its name under specified architecture
    ///
    /// This function can be useful for quickly accessing a specific module
    ///
    /// # Arguments
    /// * `name` - name of the module to find
    /// * `architecture` - architecture of the module. Should be either `ProcessInfo::proc_arch`, or `ProcessInfo::sys_arch`, or None for both.
    fn module_by_name_arch(
        &mut self,
        name: &str,
        architecture: Option<&ArchitectureIdent>,
    ) -> Result<ModuleInfo> {
        let mut ret = Err("No module found".into());
        let callback = &mut |data: ModuleInfo| {
            if data.name.as_ref() == name {
                ret = Ok(data);
                false
            } else {
                true
            }
        };
        self.module_list_callback(architecture, callback.into())?;
        ret
    }

    /// Finds any architecture process module by its name
    ///
    /// This function can be useful for quickly accessing a specific module
    ///
    /// # Arguments
    /// * `name` - name of the module to find
    fn module_by_name(&mut self, name: &str) -> Result<ModuleInfo> {
        self.module_by_name_arch(name, None)
    }

    /// Retrieves a module list for the process
    ///
    /// # Arguments
    /// * `target_arch` - sets which architecture to retrieve the modules for (if emulated). Choose
    /// between `Some(ProcessInfo::sys_arch())`, and `Some(ProcessInfo::proc_arch())`. `None` for all.
    fn module_list_arch(
        &mut self,
        target_arch: Option<&ArchitectureIdent>,
    ) -> Result<Vec<ModuleInfo>> {
        let mut ret = vec![];
        self.module_list_callback(target_arch, (&mut ret).into())?;
        Ok(ret)
    }

    /// Retrieves a module list for the process
    ///
    /// This is equivalent to `Process::module_list_arch(None)`
    fn module_list(&mut self) -> Result<Vec<ModuleInfo>> {
        self.module_list_arch(None)
    }

    /// Retrieves address of the primary module structure of the process
    ///
    /// This will generally be for the initial executable that was run
    fn primary_module_address(&mut self) -> Result<Address>;

    /// Retrieves information for the primary module of the process
    ///
    /// This will generally be the initial executable that was run
    fn primary_module(&mut self) -> Result<ModuleInfo> {
        let addr = self.primary_module_address()?;
        self.module_by_address(addr, self.info().proc_arch)
    }

    /// Retreives the process info
    fn info(&self) -> &ProcessInfo;
}

/// Process information structure
///
/// This structure implements basic process information. Architectures are provided both of the
/// system, and of the process.
#[repr(C)]
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
pub struct ProcessInfo {
    /// The base address of this process.
    ///
    /// # Remarks
    ///
    /// On Windows this will be the address of the [`_EPROCESS`](https://www.nirsoft.net/kernel_struct/vista/EPROCESS.html) structure.
    pub address: Address,
    /// ID of this process.
    pub pid: PID,
    /// Name of the process.
    pub name: ReprCStr,
    /// System architecture of the target system.
    pub sys_arch: ArchitectureIdent,
    /// Process architecture
    ///
    /// # Remarks
    ///
    /// Specifically on 64-bit systems this could be different
    /// to the `sys_arch` in case the process is an emulated 32-bit process.
    ///
    /// On windows this technique is called [`WOW64`](https://docs.microsoft.com/en-us/windows/win32/winprog64/wow64-implementation-details).
    pub proc_arch: ArchitectureIdent,
}

pub type ProcessInfoCallback<'a> = OpaqueCallback<'a, ProcessInfo>;
