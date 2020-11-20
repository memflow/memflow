use memflow::types::Address;
use memflow_ffi::process::OsProcessInfoObj;
use memflow_ffi::util::to_heap;
use memflow_win32::win32::{Win32ModuleListInfo, Win32ProcessInfo};

#[no_mangle]
pub extern "C" fn process_info_trait(
    info: &'static mut Win32ProcessInfo,
) -> &'static mut OsProcessInfoObj {
    to_heap(info)
}

#[no_mangle]
pub extern "C" fn process_info_dtb(info: &Win32ProcessInfo) -> Address {
    info.dtb
}

#[no_mangle]
pub extern "C" fn process_info_section_base(info: &Win32ProcessInfo) -> Address {
    info.section_base
}

#[no_mangle]
pub extern "C" fn process_info_exit_status(info: &Win32ProcessInfo) -> i32 {
    info.exit_status
}

#[no_mangle]
pub extern "C" fn process_info_ethread(info: &Win32ProcessInfo) -> Address {
    info.ethread
}

#[no_mangle]
pub extern "C" fn process_info_wow64(info: &Win32ProcessInfo) -> Address {
    info.wow64()
}

#[no_mangle]
pub extern "C" fn process_info_peb(info: &Win32ProcessInfo) -> Address {
    info.peb()
}

#[no_mangle]
pub extern "C" fn process_info_peb_native(info: &Win32ProcessInfo) -> Address {
    info.peb_native()
}

#[no_mangle]
pub extern "C" fn process_info_peb_wow64(info: &Win32ProcessInfo) -> Address {
    info.peb_wow64().unwrap_or_default()
}

#[no_mangle]
pub extern "C" fn process_info_teb(info: &Win32ProcessInfo) -> Address {
    info.teb.unwrap_or_default()
}

#[no_mangle]
pub extern "C" fn process_info_teb_wow64(info: &Win32ProcessInfo) -> Address {
    info.teb_wow64.unwrap_or_default()
}

#[no_mangle]
pub extern "C" fn process_info_module_info(info: &Win32ProcessInfo) -> Win32ModuleListInfo {
    info.module_info()
}

#[no_mangle]
pub extern "C" fn process_info_module_info_native(info: &Win32ProcessInfo) -> Win32ModuleListInfo {
    info.module_info_native()
}

/// Free a process information reference
///
/// # Safety
///
/// `info` must be a valid heap allocated reference to a Win32ProcessInfo structure
#[no_mangle]
pub unsafe extern "C" fn process_info_free(info: &'static mut Win32ProcessInfo) {
    let _ = Box::from_raw(info);
}
