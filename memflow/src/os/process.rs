use super::{ModuleAddressInfo, ModuleInfo, ModuleInfoCallback};
use crate::prelude::v1::{Result, *};
use std::prelude::v1::*;

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
        target_arch: Option<ArchitectureObj>,
        callback: ModuleAddressCallback<Self>,
    ) -> Result<()>;

    /// Walks the process' module list and calls the provided callback for each module
    ///
    /// # Arguments
    /// * `target_arch` - sets which architecture to retrieve the modules for (if emulated). Choose
    /// between `Some(ProcessInfo::sys_arch())`, and `Some(ProcessInfo::proc_arch())`. `None` for all.
    /// * `callback` - where to pass each matching module to. This is an opaque callback.
    fn module_list_callback(
        &mut self,
        target_arch: Option<ArchitectureObj>,
        mut callback: ModuleInfoCallback<Self>,
    ) -> Result<()> {
        let inner_callback = &mut |s: &mut Self, ModuleAddressInfo { address, arch }| match s
            .module_info_by_address(address, arch)
        {
            Ok(info) => callback.call(s, info),
            Err(e) => {
                log::trace!("Error loading module {:x} {:?}", address, e);
                false
            }
        };
        self.module_address_list_callback(target_arch, inner_callback.into())
    }

    /// Retreives a module by its structure address and architecture
    ///
    /// # Arguments
    /// * `address` - address where module's information resides in
    /// * `architecture` - architecture of the module. Should be either `ProcessInfo::proc_arch`, or `ProcessInfo::sys_arch`.
    fn module_info_by_address(
        &mut self,
        address: Address,
        architecture: ArchitectureObj,
    ) -> Result<ModuleInfo>;

    /// Retrieves a module list for the process
    ///
    /// # Arguments
    /// * `target_arch` - sets which architecture to retrieve the modules for (if emulated). Choose
    /// between `Some(ProcessInfo::sys_arch())`, and `Some(ProcessInfo::proc_arch())`. `None` for all.
    fn module_list_arch(
        &mut self,
        target_arch: Option<ArchitectureObj>,
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

    /// Retreives the process info
    fn info(&self) -> &ProcessInfo;
}

#[repr(C)]
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(::serde::Serialize))]
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
    pub sys_arch: ArchitectureObj,
    /// Process architecture
    ///
    /// # Remarks
    ///
    /// Specifically on 64-bit systems this could be different
    /// to the `sys_arch` in case the process is an emulated 32-bit process.
    ///
    /// On windows this technique is called [`WOW64`](https://docs.microsoft.com/en-us/windows/win32/winprog64/wow64-implementation-details).
    pub proc_arch: ArchitectureObj,
}

pub type ProcessInfoCallback<'a, T> = OpaqueCallback<'a, T, ProcessInfo>;
