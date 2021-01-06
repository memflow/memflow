use crate::prelude::v1::*;
use std::ffi::CString;
use std::os::raw::c_char;

#[repr(C)]
pub struct ModuleInfo {
    addr: Address,
    parent_process: Address,
    base: Address,
    size: Address,
    name: *mut c_char,
}

impl Drop for ModuleInfo {
    fn drop(&mut self) {
        let _ = unsafe { CString::from_raw(self.name) };
    }
}

pub type ModuleInfoCallback<'a> = OpaqueCallback<'a, ModuleInfo>;
