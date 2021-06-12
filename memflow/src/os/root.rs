//! Describes the root of the Operating System

use super::process::*;
use super::{AddressCallback, Process, ProcessInfo, ProcessInfoCallback};
use crate::prelude::v1::{Result, *};
use cglue::*;
use std::prelude::v1::*;

/// OS supertrait for all possible lifetimes
///
/// Use this for convenience. Chances are, once GAT are implemented, only `OS` will be kept.
///
/// It naturally provides all `OsInner` functions.
pub trait Os: for<'a> OsInner<'a> {}
impl<T: for<'a> OsInner<'a>> Os for T {}

/// High level OS trait implemented by OS layers.
///
/// This trait provides all necessary functions for handling an OS, retrieving processes, and
/// moving resources into processes.
///
/// There are also methods for accessing system level modules.
#[cglue_trait]
#[int_result]
pub trait OsInner<'a>: Send {
    #[wrap_with_obj(crate::os::process::Process)]
    type ProcessType: crate::os::process::Process + 'a;
    #[wrap_with_obj(crate::os::process::Process)]
    type IntoProcessType: crate::os::process::Process + 'static;

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
    #[skip_func]
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
    #[skip_func]
    fn process_info_by_name(&mut self, name: &str) -> Result<ProcessInfo> {
        let mut ret = Err(Error(ErrorOrigin::OsLayer, ErrorKind::ProcessNotFound));
        let callback = &mut |data: ProcessInfo| {
            if data.name.as_ref() == name {
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
    #[skip_func]
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
    fn process_by_info(&'a mut self, info: ProcessInfo) -> Result<Self::ProcessType>;
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
    #[skip_func]
    fn process_by_address(&'a mut self, addr: Address) -> Result<Self::ProcessType> {
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
    #[skip_func]
    fn process_by_name(&'a mut self, name: &str) -> Result<Self::ProcessType> {
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
    #[skip_func]
    fn process_by_pid(&'a mut self, pid: Pid) -> Result<Self::ProcessType> {
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
    #[skip_func]
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
    #[skip_func]
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
    #[skip_func]
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
    #[skip_func]
    fn module_list_callback(&mut self, mut callback: ModuleInfoCallback) -> Result<()> {
        // This is safe, because control will flow back to the callback.
        let sptr = self as *mut Self;
        let inner_callback =
            &mut |address: Address| match unsafe { &mut *sptr }.module_by_address(address) {
                Ok(info) => callback.call(info),
                Err(Error(_, ErrorKind::PartialData)) => {
                    log::trace!(
                        "Partial error when reading module {:x}, skipping entry",
                        address
                    );
                    true
                }
                Err(e) => {
                    log::trace!("Error when reading module {:x} {:?}", address, e);
                    false
                }
            };
        unsafe { sptr.as_mut().unwrap() }.module_address_list_callback(inner_callback.into())
    }

    /// Retrieves a module list for the OS
    #[skip_func]
    fn module_list(&mut self) -> Result<Vec<ModuleInfo>> {
        let mut ret = vec![];
        self.module_list_callback((&mut ret).into())?;
        Ok(ret)
    }

    /// Retrieves a module by its structure address
    ///
    /// # Arguments
    /// * `address` - address where module's information resides in
    fn module_by_address(&mut self, address: Address) -> Result<ModuleInfo>;

    /// Finds a OS module by its name
    ///
    /// This function can be useful for quickly accessing a specific module
    #[skip_func]
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
pub struct OsInfo {
    /// Base address of the OS kernel
    pub base: Address,
    /// Size of the OS kernel
    pub size: usize,
    /// System architecture
    pub arch: ArchitectureIdent,
}
