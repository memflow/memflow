use crate::util::*;
use memflow::process::*;
use std::os::raw::c_char;
use std::slice::from_raw_parts_mut;

use memflow::architecture::ArchitectureObj;
use memflow::types::Address;

pub type OsProcessInfoObj = &'static dyn OsProcessInfo;

#[no_mangle]
pub extern "C" fn os_process_info_address(obj: &OsProcessInfoObj) -> Address {
    obj.address()
}

#[no_mangle]
pub extern "C" fn os_process_info_pid(obj: &OsProcessInfoObj) -> PID {
    obj.pid()
}

/// Retreive name of the process
///
/// This will copy at most `max_len` characters (including the null terminator) into `out` of the
/// name.
///
/// # Safety
///
/// `out` must be a buffer with at least `max_len` size
#[no_mangle]
pub unsafe extern "C" fn os_process_info_name(
    obj: &OsProcessInfoObj,
    out: *mut c_char,
    max_len: usize,
) -> usize {
    let name = obj.name();
    let name_bytes = name.as_bytes();
    let out_bytes = from_raw_parts_mut(out as *mut u8, std::cmp::min(max_len, name.len() + 1));
    let len = out_bytes.len();
    out_bytes[..(len - 1)].copy_from_slice(&name_bytes[..(len - 1)]);
    *out_bytes.iter_mut().last().unwrap() = 0;
    len
}

#[no_mangle]
pub extern "C" fn os_process_info_sys_arch(obj: &OsProcessInfoObj) -> &ArchitectureObj {
    to_heap(obj.sys_arch())
}

#[no_mangle]
pub extern "C" fn os_process_info_proc_arch(obj: &OsProcessInfoObj) -> &ArchitectureObj {
    to_heap(obj.proc_arch())
}

/// Free a OsProcessInfoObj reference
///
/// # Safety
///
/// `obj` must point to a valid `OsProcessInfoObj`, and was created using one of the API's
/// functions.
#[no_mangle]
pub unsafe extern "C" fn os_process_info_free(obj: &'static mut OsProcessInfoObj) {
    let _ = Box::from_raw(obj);
}

pub type OsProcessModuleInfoObj = &'static dyn OsProcessModuleInfo;

#[no_mangle]
pub extern "C" fn os_process_module_address(obj: &OsProcessModuleInfoObj) -> Address {
    obj.address()
}

#[no_mangle]
pub extern "C" fn os_process_module_parent_process(obj: &OsProcessModuleInfoObj) -> Address {
    obj.parent_process()
}

#[no_mangle]
pub extern "C" fn os_process_module_base(obj: &OsProcessModuleInfoObj) -> Address {
    obj.base()
}

#[no_mangle]
pub extern "C" fn os_process_module_size(obj: &OsProcessModuleInfoObj) -> usize {
    obj.size()
}

/// Retreive name of the module
///
/// This will copy at most `max_len` characters (including the null terminator) into `out` of the
/// name.
///
/// # Safety
///
/// `out` must be a buffer with at least `max_len` size
#[no_mangle]
pub unsafe extern "C" fn os_process_module_name(
    obj: &OsProcessModuleInfoObj,
    out: *mut c_char,
    max_len: usize,
) -> usize {
    let name = obj.name();
    let name_bytes = name.as_bytes();
    let out_bytes = from_raw_parts_mut(out as *mut u8, std::cmp::min(max_len, name.len() + 1));
    let len = out_bytes.len();
    out_bytes[..(len - 1)].copy_from_slice(&name_bytes[..(len - 1)]);
    *out_bytes.iter_mut().last().unwrap() = 0;
    len
}

/// Free a OsProcessModuleInfoObj reference
///
/// # Safety
///
/// `obj` must point to a valid `OsProcessModuleInfoObj`, and was created using one of the API's
/// functions.
#[no_mangle]
pub unsafe extern "C" fn os_process_module_free(obj: &'static mut OsProcessModuleInfoObj) {
    let _ = Box::from_raw(obj);
}
