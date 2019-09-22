use libc::{dlsym, RTLD_DEFAULT};
use libc_print::*;
use std::ffi::CString;
use std::os::raw::{c_int, c_ulonglong, c_void};

use std::io::{Error, ErrorKind, Result};
use std::ptr::copy_nonoverlapping;

use crate::native::*;

// TODO: proper error handling
pub fn phys_read(addr: u64, len: u64) -> Result<Vec<u8>> {
    let mut l = len as c_ulonglong;
    let mem = CPU_PHYSICAL_MEMORY_MAP.unwrap()(addr, &mut l, 0);
    if mem.is_null() {
        Err(Error::new(ErrorKind::Other, "unable to read memory"))
    } else {
        let mut buf: Vec<u8> = vec![0; l as usize];
        unsafe {
            copy_nonoverlapping(mem, buf.as_mut_ptr() as *mut c_void, l as usize);
        }
        CPU_PHYSICAL_MEMORY_UNMAP.unwrap()(mem, l, 0, l);
        Ok(buf)
    }
}

pub fn phys_write(addr: u64, buf: &Vec<u8>) -> Result<u64> {
    let mut l = buf.len() as c_ulonglong;
    let mem = CPU_PHYSICAL_MEMORY_MAP.unwrap()(addr, &mut l, 1);
    if mem.is_null() {
        Err(Error::new(ErrorKind::Other, "unable to write memory"))
    } else {
        unsafe {
            copy_nonoverlapping(buf.as_ptr() as *const c_void, mem, l as usize);
        }
        CPU_PHYSICAL_MEMORY_UNMAP.unwrap()(mem, l, 1, l);
        Ok(l as u64)
    }
}
