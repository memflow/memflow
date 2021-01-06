use super::{OSProcess, ProcessInfo, ProcessInfoCallback};
use crate::prelude::v1::*;

pub trait OSKernel<T: PhysicalMemory>: Send {
    type VirtualMemoryType: VirtualMemory;
    type ProcessType: OSProcess;
    type IntoProcessType: OSProcess;

    /// Retreives physical memory object from kerenl
    fn phys_mem(&mut self) -> &mut T;

    /// Retrieves virtual memory object for the kernel memory
    fn virt_mem(&mut self) -> &mut Self::VirtualMemoryType;

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
    fn process_by_name(&mut self, name: &str) -> Result<Self::ProcessType>;
    /// Creates a process by its ID
    ///
    /// It will share the underlying memory resources
    fn process_by_pid(&mut self, pid: PID) -> Result<Self::ProcessType>;
    /// Creates a process by its internal address
    ///
    /// It will share the underlying memory resources
    fn process_by_addr(&mut self, addr: Address) -> Result<Self::ProcessType>;

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
