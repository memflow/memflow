use super::kernel::{FFIVirtualMemory, Win32Kernel};

use memflow::os::{ModuleInfo, Process};
use memflow_ffi::mem::virt_mem::VirtualMemoryObj;
use memflow_ffi::util::*;
use memflow_win32::win32::{self, Win32ProcessInfo};

use std::ffi::CStr;
use std::os::raw::c_char;

pub type Win32Process = win32::Win32Process<FFIVirtualMemory>;

/// Create a process with kernel and process info
///
/// # Safety
///
/// `kernel` must be a valid heap allocated reference to a `Kernel` object. After the function
/// call, the reference becomes invalid.
#[no_mangle]
pub unsafe extern "C" fn process_with_kernel(
    kernel: &'static mut Win32Kernel,
    proc_info: &Win32ProcessInfo,
) -> &'static mut Win32Process {
    let kernel = Box::from_raw(kernel);
    to_heap(Win32Process::with_kernel(*kernel, proc_info.clone()))
}

/// Retrieve refernce to the underlying virtual memory object
///
/// This will return a static reference to the virtual memory object. It will only be valid as long
/// as `process` if valid, and needs to be freed manually using `virt_free` regardless if the
/// process if freed or not.
#[no_mangle]
pub extern "C" fn process_virt_mem(
    process: &'static mut Win32Process,
) -> &'static mut VirtualMemoryObj {
    to_heap(&mut process.virt_mem)
}

#[no_mangle]
pub extern "C" fn process_clone(process: &Win32Process) -> &'static mut Win32Process {
    to_heap((*process).clone())
}

/// Frees the `process`
///
/// # Safety
///
/// `process` must be a valid heap allocated reference to a `Win32Process` object. After the
/// function returns, the reference becomes invalid.
#[no_mangle]
pub unsafe extern "C" fn process_free(process: &'static mut Win32Process) {
    let _ = Box::from_raw(process);
}

/// Retrieve a process module list
///
/// This will fill up to `max_len` elements into `out` with `ModuleInfo` objects.
///
/// # Safety
///
/// `out` must be a valid buffer able to contain `max_len` `ModuleInfo` objects.
#[no_mangle]
pub unsafe extern "C" fn process_module_list(
    process: &mut Win32Process,
    out: *mut ModuleInfo,
    max_len: usize,
) -> usize {
    let mut ret = 0;

    let buffer = std::slice::from_raw_parts_mut(out, max_len);

    let callback = &mut |_: &mut _, info| {
        if ret < max_len {
            buffer[ret] = info;
            ret += 1;
            true
        } else {
            false
        }
    };

    process
        .module_list_callback(None, callback.into())
        .map_err(inspect_err)
        .ok()
        .map(|_| ret)
        .unwrap_or_default()
}

/// Retrieve the main module of the process
///
/// This function searches for a module with a base address
/// matching the section_base address from the ProcessInfo structure.
/// It then writes a `ModuleInfo` object into the address given, and
/// returns `0`, on error, `-1` is returned.
#[no_mangle]
pub extern "C" fn process_main_module_info(
    process: &mut Win32Process,
    output: &mut ModuleInfo,
) -> i32 {
    if let Ok(m) = process.main_module_info().map_err(inspect_err) {
        *output = m;
        0
    } else {
        -1
    }
}

/// Lookup a module
///
/// This will search for a module called `name`, and write the `ModuleInfo` object
/// if it was found, and return `0`. On error, `-1` is returned
///
/// # Safety
///
/// `name` must be a valid null terminated string.
#[no_mangle]
pub unsafe extern "C" fn process_module_info(
    process: &mut Win32Process,
    name: *const c_char,
    output: &mut ModuleInfo,
) -> i32 {
    let name = CStr::from_ptr(name).to_string_lossy();

    if let Ok(m) = process.module_info(&name).map_err(inspect_err) {
        *output = m;
        0
    } else {
        -1
    }
}
