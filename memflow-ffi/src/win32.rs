/*
use std::ffi::c_void;
use std::ptr;

use memflow_win32::*;
*/

/*
/// # Safety
///
/// this function has to be called with an initialized memory backend
/// this function will return a pointer to a win32 object that has to be freed via win32_free()
#[no_mangle]
pub unsafe extern "C" fn win32_init(mem: *mut c_void) -> *mut Win32 {
    if !mem.is_null() {
        let mut _mem: Box<Box<dyn MemoryBackend>> = std::mem::transmute(mem as *mut _);

        let _os = Win32::try_with(&mut **_mem).unwrap();

        Box::leak(_mem);
        return std::mem::transmute(Box::new(_os));
    }

    ptr::null_mut()
}

/// # Safety
///
/// this function has to be called with a pointer that has been initialized from win32_init()
#[no_mangle]
pub unsafe extern "C" fn win32_free(win32: *mut Win32) {
    if !win32.is_null() {
        let _win32: Box<Win32> = std::mem::transmute(win32);
        // drop _win32
    }
}

/// # Safety
///
/// this function will return a pointer to a win32_offsets object that has to be freed via win32_offsets_free()
#[no_mangle]
pub unsafe extern "C" fn win32_offsets_init(win32: *mut Win32) -> *mut Win32Offsets {
    if !win32.is_null() {
        let _win32: Box<Win32> = std::mem::transmute(win32);

        let _offsets = Win32Offsets::try_with_guid(&_win32.kernel_guid()).unwrap();

        Box::leak(_win32);
        return std::mem::transmute(Box::new(_offsets));
    }

    ptr::null_mut()
}

/// # Safety
///
/// this function has to be called with a pointer that has been initialized from win32_offsets_init()
#[no_mangle]
pub unsafe extern "C" fn win32_offsets_free(offsets: *mut Win32Offsets) {
    if !offsets.is_null() {
        let _offsets: Box<Win32Offsets> = std::mem::transmute(offsets);
        // drop _offsets
    }
}
*/
