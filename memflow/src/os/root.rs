//! Describes the root of the Operating System

use super::process::*;
use super::{AddressCallback, ProcessInfo, ProcessInfoCallback};

use crate::prelude::v1::{Result, *};

use crate::cglue::*;
use std::prelude::v1::*;

/// High level OS trait implemented by OS layers.
///
/// This trait provides all necessary functions for handling an OS, retrieving processes, and
/// moving resources into processes.
///
/// There are also methods for accessing system level modules.
#[cfg_attr(feature = "plugins", cglue_trait)]
#[int_result]
pub trait Os: Send {
    #[wrap_with_group(crate::plugins::os::ProcessInstance)]
    type ProcessType<'a>: crate::os::process::Process + MemoryView + 'a
    where
        Self: 'a;
    #[wrap_with_group(crate::plugins::os::IntoProcessInstance)]
    type IntoProcessType: crate::os::process::Process + MemoryView + Clone + 'static;

    /// Walks a process list and calls a callback for each process structure address
    ///
    /// The callback is fully opaque. We need this style so that C FFI can work seamlessly.
    fn process_address_list_callback(&mut self, callback: AddressCallback) -> Result<()>;

    /// Retrieves a process address list
    ///
    /// This will be a list of unique internal addresses for underlying process structures
    #[skip_func]
    fn process_address_list(&mut self) -> Result<Vec<Address>> {
        let mut ret = vec![];
        self.process_address_list_callback((&mut ret).into())?;
        Ok(ret)
    }

    /// Walks a process list and calls a callback for each process
    ///
    /// The callback is fully opaque. We need this style so that C FFI can work seamlessly.
    fn process_info_list_callback(&mut self, mut callback: ProcessInfoCallback) -> Result<()> {
        // This is safe, because control will flow back to the callback.
        let sptr = self as *mut Self;
        let inner_callback = &mut |addr| match unsafe { &mut *sptr }.process_info_by_address(addr) {
            Ok(info) => callback.call(info),
            Err(Error(_, ErrorKind::PartialData)) => {
                log::trace!("Partial error when reading process {:x}", addr);
                true
            }
            Err(e) => {
                log::trace!("Error when reading process {:x} {:?}", addr, e);
                false
            }
        };
        unsafe { sptr.as_mut().unwrap() }.process_address_list_callback(inner_callback.into())
    }

    /// Retrieves a process list
    #[skip_func]
    fn process_info_list(&mut self) -> Result<Vec<ProcessInfo>> {
        let mut ret = vec![];
        self.process_info_list_callback((&mut ret).into())?;
        Ok(ret)
    }

    /// Find process information by its internal address
    fn process_info_by_address(&mut self, address: Address) -> Result<ProcessInfo>;

    /// Find process information by its name
    ///
    /// # Remarks:
    ///
    /// This function only returns processes whose state is not [`ProcessState::Dead`].
    fn process_info_by_name(&mut self, name: &str) -> Result<ProcessInfo> {
        let mut ret = Err(Error(ErrorOrigin::OsLayer, ErrorKind::ProcessNotFound));
        let callback = &mut |data: ProcessInfo| {
            if (data.state == ProcessState::Unknown || data.state == ProcessState::Alive)
                && data.name.as_ref() == name
            {
                ret = Ok(data);
                false
            } else {
                true
            }
        };
        self.process_info_list_callback(callback.into())?;
        ret
    }

    /// Find process information by its ID
    fn process_info_by_pid(&mut self, pid: Pid) -> Result<ProcessInfo> {
        let mut ret = Err(Error(ErrorOrigin::OsLayer, ErrorKind::ProcessNotFound));
        let callback = &mut |data: ProcessInfo| {
            if data.pid == pid {
                ret = Ok(data);
                false
            } else {
                true
            }
        };
        self.process_info_list_callback(callback.into())?;
        ret
    }

    /// Construct a process by its info, borrowing the OS
    ///
    /// It will share the underlying memory resources
    fn process_by_info(&mut self, info: ProcessInfo) -> Result<Self::ProcessType<'_>>;

    /// Construct a process by its info, consuming the OS
    ///
    /// This function will consume the Kernel instance and move its resources into the process
    fn into_process_by_info(self, info: ProcessInfo) -> Result<Self::IntoProcessType>;

    /// Creates a process by its internal address, borrowing the OS
    ///
    /// It will share the underlying memory resources
    ///
    /// If no process with the specified address can be found this function will return an Error.
    ///
    /// This function can be useful for quickly accessing a process.
    fn process_by_address(&mut self, addr: Address) -> Result<Self::ProcessType<'_>> {
        self.process_info_by_address(addr)
            .and_then(move |i| self.process_by_info(i))
    }

    /// Creates a process by its name, borrowing the OS
    ///
    /// It will share the underlying memory resources
    ///
    /// If no process with the specified name can be found this function will return an Error.
    ///
    /// This function can be useful for quickly accessing a process.
    ///
    /// # Remarks:
    ///
    /// This function only returns processes whose state is not [`ProcessState::Dead`].
    fn process_by_name(&mut self, name: &str) -> Result<Self::ProcessType<'_>> {
        self.process_info_by_name(name)
            .and_then(move |i| self.process_by_info(i))
    }

    /// Creates a process by its ID, borrowing the OS
    ///
    /// It will share the underlying memory resources
    ///
    /// If no process with the specified ID can be found this function will return an Error.
    ///
    /// This function can be useful for quickly accessing a process.
    fn process_by_pid(&mut self, pid: Pid) -> Result<Self::ProcessType<'_>> {
        self.process_info_by_pid(pid)
            .and_then(move |i| self.process_by_info(i))
    }

    /// Creates a process by its internal address, consuming the OS
    ///
    /// It will consume the OS and not affect memory usage
    ///
    /// If no process with the specified address can be found this function will return an Error.
    ///
    /// This function can be useful for quickly accessing a process.
    fn into_process_by_address(mut self, addr: Address) -> Result<Self::IntoProcessType>
    where
        Self: Sized,
    {
        self.process_info_by_address(addr)
            .and_then(|i| self.into_process_by_info(i))
    }

    /// Creates a process by its name, consuming the OS
    ///
    /// It will consume the OS and not affect memory usage
    ///
    /// If no process with the specified name can be found this function will return an Error.
    ///
    /// This function can be useful for quickly accessing a process.
    ///
    /// # Remarks:
    ///
    /// This function only returns processes whose state is not [`ProcessState::Dead`].
    fn into_process_by_name(mut self, name: &str) -> Result<Self::IntoProcessType>
    where
        Self: Sized,
    {
        self.process_info_by_name(name)
            .and_then(|i| self.into_process_by_info(i))
    }

    /// Creates a process by its ID, consuming the OS
    ///
    /// It will consume the OS and not affect memory usage
    ///
    /// If no process with the specified ID can be found this function will return an Error.
    ///
    /// This function can be useful for quickly accessing a process.
    fn into_process_by_pid(mut self, pid: Pid) -> Result<Self::IntoProcessType>
    where
        Self: Sized,
    {
        self.process_info_by_pid(pid)
            .and_then(|i| self.into_process_by_info(i))
    }

    /// Walks the OS module list and calls the provided callback for each module structure
    /// address
    ///
    /// # Arguments
    /// * `callback` - where to pass each matching module to. This is an opaque callback.
    fn module_address_list_callback(&mut self, callback: AddressCallback) -> Result<()>;

    /// Walks the OS module list and calls the provided callback for each module
    ///
    /// # Arguments
    /// * `callback` - where to pass each matching module to. This is an opaque callback.
    fn module_list_callback(&mut self, mut callback: ModuleInfoCallback) -> Result<()> {
        // This is safe, because control will flow back to the callback.
        let sptr = self as *mut Self;
        let inner_callback =
            &mut |address: Address| match unsafe { &mut *sptr }.module_by_address(address) {
                Ok(info) => callback.call(info),
                Err(e) => {
                    log::trace!("Error when reading module {:x} {:?}", address, e);
                    true // continue iteration
                }
            };
        unsafe { sptr.as_mut().unwrap() }.module_address_list_callback(inner_callback.into())
    }

    /// Retrieves a module by its structure address
    ///
    /// # Arguments
    /// * `address` - address where module's information resides in
    fn module_by_address(&mut self, address: Address) -> Result<ModuleInfo>;

    /// Finds a OS module by its name
    ///
    /// This function can be useful for quickly accessing a specific module
    fn module_by_name(&mut self, name: &str) -> Result<ModuleInfo> {
        let mut ret = Err(Error(ErrorOrigin::OsLayer, ErrorKind::ProcessNotFound));
        let callback = &mut |data: ModuleInfo| {
            if data.name.as_ref() == name {
                ret = Ok(data);
                false
            } else {
                true
            }
        };
        self.module_list_callback(callback.into())?;
        ret
    }

    /// Retrieves a module list for the OS
    #[skip_func]
    fn module_list(&mut self) -> Result<Vec<ModuleInfo>> {
        let mut ret = vec![];
        self.module_list_callback((&mut ret).into())?;
        Ok(ret)
    }

    /// Retrieves address of the primary module of the OS
    ///
    /// This will generally be for the main kernel process/module
    fn primary_module_address(&mut self) -> Result<Address>;

    /// Retrieves information for the primary module of the OS
    ///
    /// This will generally be for the main kernel process/module
    fn primary_module(&mut self) -> Result<ModuleInfo> {
        let addr = self.primary_module_address()?;
        self.module_by_address(addr)
    }

    /// Retrieves a list of all imports of a given module
    fn module_import_list_callback(
        &mut self,
        info: &ModuleInfo,
        callback: ImportCallback,
    ) -> Result<()>;

    /// Retrieves a list of all exports of a given module
    fn module_export_list_callback(
        &mut self,
        info: &ModuleInfo,
        callback: ExportCallback,
    ) -> Result<()>;

    /// Retrieves a list of all sections of a given module
    fn module_section_list_callback(
        &mut self,
        info: &ModuleInfo,
        callback: SectionCallback,
    ) -> Result<()>;

    /// Retrieves a list of all imports of a given module
    #[skip_func]
    fn module_import_list(&mut self, info: &ModuleInfo) -> Result<Vec<ImportInfo>> {
        let mut ret = vec![];
        self.module_import_list_callback(info, (&mut ret).into())?;
        Ok(ret)
    }

    /// Retrieves a list of all exports of a given module
    #[skip_func]
    fn module_export_list(&mut self, info: &ModuleInfo) -> Result<Vec<ExportInfo>> {
        let mut ret = vec![];
        self.module_export_list_callback(info, (&mut ret).into())?;
        Ok(ret)
    }

    /// Retrieves a list of all sections of a given module
    #[skip_func]
    fn module_section_list(&mut self, info: &ModuleInfo) -> Result<Vec<SectionInfo>> {
        let mut ret = vec![];
        self.module_section_list_callback(info, (&mut ret).into())?;
        Ok(ret)
    }

    /// Finds a single import of a given module by its name
    fn module_import_by_name(&mut self, info: &ModuleInfo, name: &str) -> Result<ImportInfo> {
        let mut ret = Err(Error(ErrorOrigin::OsLayer, ErrorKind::ImportNotFound));
        let callback = &mut |data: ImportInfo| {
            if data.name.as_ref() == name {
                ret = Ok(data);
                false
            } else {
                true
            }
        };
        self.module_import_list_callback(info, callback.into())?;
        ret
    }

    /// Finds a single export of a given module by its name
    fn module_export_by_name(&mut self, info: &ModuleInfo, name: &str) -> Result<ExportInfo> {
        let mut ret = Err(Error(ErrorOrigin::OsLayer, ErrorKind::ImportNotFound));
        let callback = &mut |data: ExportInfo| {
            if data.name.as_ref() == name {
                ret = Ok(data);
                false
            } else {
                true
            }
        };
        self.module_export_list_callback(info, callback.into())?;
        ret
    }

    /// Finds a single section of a given module by its name
    fn module_section_by_name(&mut self, info: &ModuleInfo, name: &str) -> Result<SectionInfo> {
        let mut ret = Err(Error(ErrorOrigin::OsLayer, ErrorKind::ImportNotFound));
        let callback = &mut |data: SectionInfo| {
            if data.name.as_ref() == name {
                ret = Ok(data);
                false
            } else {
                true
            }
        };
        self.module_section_list_callback(info, callback.into())?;
        ret
    }

    /// Retrieves the OS info
    fn info(&self) -> &OsInfo;
}

/// Information block about OS
///
/// This provides some basic information about the OS in question. `base`, and `size` may be
/// omitted in some circumstances (lack of kernel, or privileges). But architecture should always
/// be correct.
#[repr(C)]
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[cfg_attr(feature = "abi_stable", derive(::abi_stable::StableAbi))]
pub struct OsInfo {
    /// Base address of the OS kernel
    pub base: Address,
    /// Size of the OS kernel
    pub size: umem,
    /// System architecture
    pub arch: ArchitectureIdent,
}
