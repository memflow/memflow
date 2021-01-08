use super::{Process, ProcessInfo, ProcessInfoCallback};
use crate::prelude::v1::{Result, *};
use std::prelude::v1::*;

pub trait Kernel<'a>: Send {
    type PhysicalMemoryType: PhysicalMemory + 'a;
    type VirtualMemoryType: VirtualMemory + 'a;
    type ProcessType: Process + 'a;
    type IntoProcessType: Process;

    /// Retreives physical memory object from kernel
    fn phys_mem(&'a mut self) -> Self::PhysicalMemoryType;

    /// Retrieves virtual memory object for the kernel memory
    fn virt_mem(&'a mut self) -> Self::VirtualMemoryType;

    /// Walks a process list and calls a callback for each process
    ///
    /// The callback is fully opaque. We need this style so that C FFI can work seamlessly.
    fn process_list_callback(&mut self, callback: ProcessInfoCallback) -> Result<()>;

    /// Retreives a process list
    fn process_list(&mut self) -> Result<Vec<ProcessInfo>> {
        let mut ret = vec![];
        let callback = &mut |data| ret.push(data);
        self.process_list_callback(callback.into())?;
        Ok(ret)
    }

    /// Find process information by its internal address
    fn process_info_by_address(&mut self, address: Address) -> Result<ProcessInfo>;

    /// Find process information by its name
    fn process_info_by_name(&mut self, name: &str) -> Result<ProcessInfo> {
        let mut ret = Err("No process found".into());
        let callback = &mut |data: ProcessInfo| {
            if ret.is_err() && data.name.as_ref() == name {
                ret = Ok(data)
            }
        };
        self.process_list_callback(callback.into())?;
        ret
    }

    /// Find process information by its ID
    fn process_info_by_pid(&mut self, pid: PID) -> Result<ProcessInfo> {
        let mut ret = Err("No process found".into());
        let callback = &mut |data: ProcessInfo| {
            if data.pid == pid {
                ret = Ok(data)
            }
        };
        self.process_list_callback(callback.into())?;
        ret
    }

    /// Construct a process by its info
    ///
    /// It will share the underlying memory resources
    fn process_by_info(&'a mut self, info: ProcessInfo) -> Result<Self::ProcessType>;
    /// Construct a process by its info
    ///
    /// This function will consume the Kernel instance and move its resources into the process
    fn into_process_by_info(self, info: ProcessInfo) -> Result<Self::IntoProcessType>;

    /// Creates a process by its internal address
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
    /// Creates a process by its name
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
    /// Creates a process by its ID
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

    /// Creates a process by its internal address
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
    /// Creates a process by its name
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
    /// Creates a process by its ID
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
