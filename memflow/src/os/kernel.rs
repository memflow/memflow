use super::{AddressCallback, Process, ProcessInfo, ProcessInfoCallback};
use crate::prelude::v1::{Result, *};
use std::prelude::v1::*;

pub trait Kernel<'a>: Send {
    type ProcessType: Process + 'a;
    type IntoProcessType: Process;

    /// Walks a process list and calls a callback for each process structure address
    ///
    /// The callback is fully opaque. We need this style so that C FFI can work seamlessly.
    fn process_address_list_callback(&mut self, callback: AddressCallback<Self>) -> Result<()>;

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
    fn process_info_list_callback(
        &mut self,
        mut callback: ProcessInfoCallback<Self>,
    ) -> Result<()> {
        let inner_callback = &mut |s: &mut Self, addr| match s.process_info_by_address(addr) {
            Ok(info) => callback.call(s, info),
            Err(e) => {
                log::trace!("Failed to read process {:x} {:?}", addr, e);
                false
            }
        };
        self.process_address_list_callback(inner_callback.into())
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
        let callback = &mut |_: &mut Self, data: ProcessInfo| {
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
        let callback = &mut |_: &mut Self, data: ProcessInfo| {
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

    /// Construct a kernel address space process, borrowing the kernel
    ///
    /// It will share the underlying memory resources
    fn kernel_process(&'a mut self) -> Result<Self::ProcessType>;
    /// Construct a kernel address space process, consuming the kernel
    ///
    /// This function will consume the Kernel instance and move its resources into the process
    fn into_kernel_process(self) -> Result<Self::IntoProcessType>
    where
        Self: Sized;

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
}
