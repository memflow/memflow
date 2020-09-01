use memflow_ffi::process::OsProcessModuleInfoObj;
use memflow_ffi::util::to_heap;
use memflow_win32::win32::Win32ModuleInfo;

#[no_mangle]
pub extern "C" fn module_info_trait(
    info: &'static mut Win32ModuleInfo,
) -> &'static mut OsProcessModuleInfoObj {
    to_heap(info)
}

/// Free a win32 module info instance.
///
/// Note that it is not the same as `OsProcessModuleInfoObj`, and those references need to be freed
/// manually.
///
/// # Safety
///
/// `info` must be a unique heap allocated reference to `Win32ModuleInfo`, and after this call the
/// reference will become invalid.
#[no_mangle]
pub unsafe extern "C" fn module_info_free(info: &'static mut Win32ModuleInfo) {
    let _ = Box::from_raw(info);
}
