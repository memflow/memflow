use super::mem::MemoryBackend;

use std::ffi::c_void;
use std::ptr;

use flow_qemu_procfs::Memory;

/// # Safety
///
/// this function might return a null pointer when the qemu_procfs backend cannot be initialized
#[no_mangle]
pub unsafe extern "C" fn qemu_procfs_init() -> *mut c_void {
    match Memory::new() {
        Ok(m) => {
            let inner: Box<dyn MemoryBackend> = Box::new(m);
            Box::into_raw(Box::new(inner)) as *mut c_void
        }
        Err(_) => ptr::null_mut(),
    }
}

/// # Safety
///
/// this function has to be called with an initialized qemu_procfs backend
#[no_mangle]
pub unsafe extern "C" fn qemu_procfs_free(ptr: *mut c_void) {
    if !ptr.is_null() {
        let mut _mem: Box<Box<dyn MemoryBackend>> = std::mem::transmute(ptr as *mut _);
        // drop _mem
    }
}
