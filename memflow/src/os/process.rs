use super::{ModuleInfo, ModuleInfoCallback};
use crate::prelude::v1::*;
use std::ffi::CString;
use std::os::raw::c_char;

pub trait OSProcess: Send {
    type VirtualMemoryType: VirtualMemory;
    type VirtualTranslateType: VirtualTranslate;

    /// Type if the process gets cloned
    ///
    /// This is important, since cloning a process which merely references physical memory, and
    /// virtual translation objects is impossible, without it changing types
    type ClonedProcessType: OSProcess;

    /// Retrieves virtual memory object for the process
    fn virt_mem(&mut self) -> &mut Self::VirtualMemoryType;

    /// Retrieves virtual address translator for the process (if applicable)
    fn vat(&mut self) -> Option<&mut Self::VirtualTranslateType>;

    /// Clone the process
    fn clone_process(&self) -> Self::ClonedProcessType;

    /// Walks the process' module list and calls the provided callback for each module
    fn module_list_callback(&mut self, callback: ModuleInfoCallback) -> Result<()>;

    /// Retrieves a module list for the process
    fn module_list(&mut self) -> Result<Vec<ModuleInfo>> {
        let mut ret = vec![];
        let callback = &mut |data| ret.push(data);
        self.module_list_callback(callback.into())?;
        Ok(ret)
    }
}

#[repr(C)]
pub struct ProcessInfo {
    addr: Address,
    pid: PID,
    name: *mut c_char,
    sys_arch: ArchitectureObj,
    proc_arch: ArchitectureObj,
}

impl Drop for ProcessInfo {
    fn drop(&mut self) {
        let _ = unsafe { CString::from_raw(self.name) };
    }
}

pub type ProcessInfoCallback<'a> = OpaqueCallback<'a, ProcessInfo>;
