use super::kernel::{FFIVirtualMemory, Kernel};

use memflow::iter::FnExtend;
use memflow_ffi::mem::virt_mem::VirtualMemoryObj;
use memflow_ffi::util::*;
use memflow_win32::win32::{self, Win32ModuleInfo, Win32ProcessInfo};

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
    kernel: &'static mut Kernel,
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
/// This will fill up to `max_len` elements into `out` with references to `Win32ModuleInfo` objects.
///
/// These references then need to be freed with `module_info_free`
///
/// # Safety
///
/// `out` must be a valid buffer able to contain `max_len` references to `Win32ModuleInfo`.
#[no_mangle]
pub unsafe extern "C" fn process_module_list(
    process: &mut Win32Process,
    out: *mut &'static mut Win32ModuleInfo,
    max_len: usize,
) -> usize {
    let mut ret = 0;

    let buffer = std::slice::from_raw_parts_mut(out, max_len);

    let mut extend_fn = FnExtend::new(|info| {
        if ret < max_len {
            buffer[ret] = to_heap(info);
            ret += 1;
        }
    });

    process
        .module_list_extend(&mut extend_fn)
        .map_err(inspect_err)
        .ok()
        .map(|_| ret)
        .unwrap_or_default()
}

/// Retrieve the main module of the process
///
/// This function searches for a module with a base address
/// matching the section_base address from the ProcessInfo structure.
/// It then returns a reference to a newly allocated
/// `Win32ModuleInfo` object, if a module was found (null otherwise).
///
/// The reference later needs to be freed with `module_info_free`
///
/// # Safety
///
/// `process` must be a valid Win32Process pointer.
#[no_mangle]
pub unsafe extern "C" fn process_main_module_info(
    process: &mut Win32Process,
) -> Option<&'static mut Win32ModuleInfo> {
    process
        .main_module_info()
        .map(to_heap)
        .map_err(inspect_err)
        .ok()
}

/// Lookup a module
///
/// This will search for a module called `name`, and return a reference to a newly allocated
/// `Win32ModuleInfo` object, if a module was found (null otherwise).
///
/// The reference later needs to be freed with `module_info_free`
///
/// # Safety
///
/// `process` must be a valid Win32Process pointer.
/// `name` must be a valid null terminated string.
#[no_mangle]
pub unsafe extern "C" fn process_module_info(
    process: &mut Win32Process,
    name: *const c_char,
) -> Option<&'static mut Win32ModuleInfo> {
    let name = CStr::from_ptr(name).to_string_lossy();

    process
        .module_info(&name)
        .map(to_heap)
        .map_err(inspect_err)
        .ok()
}
