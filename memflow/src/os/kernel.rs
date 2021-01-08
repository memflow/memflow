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

    /// Creates a process by its name
    ///
    /// It will share the underlying memory resources
    fn process_by_name(&'a mut self, name: &str) -> Result<Self::ProcessType>;
    /// Creates a process by its ID
    ///
    /// It will share the underlying memory resources
    fn process_by_pid(&'a mut self, pid: PID) -> Result<Self::ProcessType>;
    /// Creates a process by its internal address
    ///
    /// It will share the underlying memory resources
    fn process_by_addr(&'a mut self, addr: Address) -> Result<Self::ProcessType>;

    /// Creates a process by its name
    ///
    /// It will consume the kernel and not affect memory usage
    fn into_process_by_name(self, name: &str) -> Result<Self::IntoProcessType>;
    /// Creates a process by its ID
    ///
    /// It will consume the kernel and not affect memory usage
    fn into_process_by_pid(self, pid: PID) -> Result<Self::IntoProcessType>;
    /// Creates a process by its internal address
    ///
    /// It will consume the kernel and not affect memory usage
    fn into_process_by_addr(self, addr: Address) -> Result<Self::IntoProcessType>;
}
