use super::{AddressCallback, Process, ProcessInfo, ProcessInfoCallback};
use crate::prelude::v1::{Result, *};
use std::prelude::v1::*;

/// Kernel supertrait for all possible lifetimes
pub trait Kernel: for<'a> KernelInner<'a> {}
impl<T: for<'a> KernelInner<'a>> Kernel for T {}

pub trait KernelInner<'a>: Send {
    type ProcessType: Process + 'a;
    type IntoProcessType: Process + Clone;

    /// Walks a process list and calls a callback for each process structure address
    ///
    /// The callback is fully opaque. We need this style so that C FFI can work seamlessly.
    fn process_address_list_callback(&mut self, callback: AddressCallback) -> Result<()>;

    /// Retrieves a process address list
    ///
    /// This will be a list of unique internal addresses for underlying process structures
    fn process_address_list(&mut self) -> Result<Vec<Address>> {
        let mut ret = vec![];
        self.process_address_list_callback((&mut ret).into())?;
        Ok(ret)
    }

    /// Walks a process list and calls a callback for each process
    ///
    /// The callback is fully opaque. We need this style so that C FFI can work seamlessly.
    fn process_info_list_callback(&mut self, mut callback: ProcessInfoCallback) -> Result<()> {
        let (s1, s2) = unsafe { super::clone_self(self) };
        let inner_callback = &mut |addr| match s2.process_info_by_address(addr) {
            Ok(info) => callback.call(info),
            Err(e) => {
                log::trace!("Failed to read process {:x} {:?}", addr, e);
                false
            }
        };
        s1.process_address_list_callback(inner_callback.into())
    }

    /// Retrieves a process list
    fn process_info_list(&mut self) -> Result<Vec<ProcessInfo>> {
        let mut ret = vec![];
        self.process_info_list_callback((&mut ret).into())?;
        Ok(ret)
    }

    /// Find process information by its internal address
    fn process_info_by_address(&mut self, address: Address) -> Result<ProcessInfo>;

    /// Find process information by its name
    fn process_info_by_name(&mut self, name: &str) -> Result<ProcessInfo> {
        let mut ret = Err("No process found".into());
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
    fn process_info_by_pid(&mut self, pid: PID) -> Result<ProcessInfo> {
        let mut ret = Err("No process found".into());
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

    /// Construct a process by its info, borrowing the kernel
    ///
    /// It will share the underlying memory resources
    fn process_by_info(&'a mut self, info: ProcessInfo) -> Result<Self::ProcessType>;
    /// Construct a process by its info, consuming the kernel
    ///
    /// This function will consume the Kernel instance and move its resources into the process
    fn into_process_by_info(self, info: ProcessInfo) -> Result<Self::IntoProcessType>;

    /// Creates a process by its internal address, borrowing the kernel
    ///
    /// It will share the underlying memory resources
    ///
    /// If no process with the specified address can be found this function will return an Error.
    ///
    /// This function can be useful for quickly accessing a process.
    fn process_by_address(&'a mut self, addr: Address) -> Result<Self::ProcessType> {
        self.process_info_by_address(addr)
            .and_then(move |i| self.process_by_info(i))
    }
    /// Creates a process by its name, borrowing the kernel
    ///
    /// It will share the underlying memory resources
    ///
    /// If no process with the specified name can be found this function will return an Error.
    ///
    /// This function can be useful for quickly accessing a process.
    fn process_by_name(&'a mut self, name: &str) -> Result<Self::ProcessType> {
        self.process_info_by_name(name)
            .and_then(move |i| self.process_by_info(i))
    }
    /// Creates a process by its ID, borrowing the kernel
    ///
    /// It will share the underlying memory resources
    ///
    /// If no process with the specified ID can be found this function will return an Error.
    ///
    /// This function can be useful for quickly accessing a process.
    fn process_by_pid(&'a mut self, pid: PID) -> Result<Self::ProcessType> {
        self.process_info_by_pid(pid)
            .and_then(move |i| self.process_by_info(i))
    }

    /// Creates a process by its internal address, consuming the kernel
    ///
    /// It will consume the kernel and not affect memory usage
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
    /// Creates a process by its name, consuming the kernel
    ///
    /// It will consume the kernel and not affect memory usage
    ///
    /// If no process with the specified name can be found this function will return an Error.
    ///
    /// This function can be useful for quickly accessing a process.
    fn into_process_by_name(mut self, name: &str) -> Result<Self::IntoProcessType>
    where
        Self: Sized,
    {
        self.process_info_by_name(name)
            .and_then(|i| self.into_process_by_info(i))
    }
    /// Creates a process by its ID, consuming the kernel
    ///
    /// It will consume the kernel and not affect memory usage
    ///
    /// If no process with the specified ID can be found this function will return an Error.
    ///
    /// This function can be useful for quickly accessing a process.
    fn into_process_by_pid(mut self, pid: PID) -> Result<Self::IntoProcessType>
    where
        Self: Sized,
    {
        self.process_info_by_pid(pid)
            .and_then(|i| self.into_process_by_info(i))
    }

    /// Walks the kernel module list and calls the provided callback for each module structure
    /// address
    ///
    /// # Arguments
    /// * `callback` - where to pass each matching module to. This is an opaque callback.
    fn module_address_list_callback(&mut self, callback: AddressCallback) -> Result<()>;

    /// Walks the kernel module list and calls the provided callback for each module
    ///
    /// # Arguments
    /// * `callback` - where to pass each matching module to. This is an opaque callback.
    fn module_list_callback(&mut self, mut callback: ModuleInfoCallback) -> Result<()> {
        let (s1, s2) = unsafe { super::clone_self(self) };
        let inner_callback = &mut |address: Address| match s2.module_by_address(address) {
            Ok(info) => callback.call(info),
            Err(e) => {
                log::trace!("Error loading module {:x} {:?}", address, e);
                false
            }
        };
        s1.module_address_list_callback(inner_callback.into())
    }

    /// Retrieves a module list for the kernel
    fn module_list(&mut self) -> Result<Vec<ModuleInfo>> {
        let mut ret = vec![];
        self.module_list_callback((&mut ret).into())?;
        Ok(ret)
    }

    /// Retreives a module by its structure address
    ///
    /// # Arguments
    /// * `address` - address where module's information resides in
    fn module_by_address(&mut self, address: Address) -> Result<ModuleInfo>;

    /// Finds a kernel module by its name
    ///
    /// This function can be useful for quickly accessing a specific module
    fn module_by_name(&mut self, name: &str) -> Result<ModuleInfo> {
        let mut ret = Err("No module found".into());
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

    /// Retreives the kernel info
    fn info(&self) -> &KernelInfo;
}

#[repr(C)]
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(::serde::Serialize))]
pub struct KernelInfo {
    /// Base address of the kernel
    pub base: Address,
    /// Size of the kernel
    pub size: usize,
    /// System architecture
    pub arch: ArchitectureIdent,
}
