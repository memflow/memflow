use std::ffi::{c_void, CStr};
use std::os::raw::c_char;
use std::ptr;

use memflow_core::mem::PhysicalMemory;
use memflow_coredump::create_connector;

/// # Safety
///
/// this function might return a null pointer when the qemu_procfs backend cannot be initialized
#[no_mangle]
pub unsafe extern "C" fn coredump_open(path: *const c_char) -> *mut c_void {
    if path.is_null() {
        return ptr::null_mut();
    }

    let c_path = CStr::from_ptr(path);
    let pathbuf = c_path.to_string_lossy();
    match create_connector(&pathbuf) {
        Ok(m) => {
            let inner: Box<dyn PhysicalMemory> = Box::new(m);
            Box::into_raw(Box::new(inner)) as *mut c_void
        }
        Err(_) => ptr::null_mut(),
    }
}

/// # Safety
///
/// this function has to be called with an initialized coredump backend
#[no_mangle]
pub unsafe extern "C" fn coredump_free(ptr: *mut c_void) {
    if !ptr.is_null() {
        let mut _mem: Box<Box<dyn PhysicalMemory>> = std::mem::transmute(ptr as *mut _);
        // drop _mem
    }
}
