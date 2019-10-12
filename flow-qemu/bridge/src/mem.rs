use std::os::raw::{c_ulonglong, c_void};

use std::io::{Error, ErrorKind, Result};
use std::ptr::copy_nonoverlapping;

use ::mem::{PhysicalRead, PhysicalWrite};
use address::{Address, Length};

use std::ffi::CString;

use crate::native::*;

pub struct Wrapper;

impl Wrapper {
    pub fn new() -> Wrapper {
        if cfg!(feature = "locking") {
            // we will lock the main thread of qemu to prevent writes here
            // this will encapsulte either physical memory or vat reads
            let file_cstr = CString::new(file!()).unwrap();
            QEMU_MUTEX_LOCK_IOTHREAD_IMPL.unwrap()(file_cstr.as_ptr(), line!() as i32);
        }
        Wrapper {}
    }
}

#[cfg(feature = "locking")]
impl Drop for Wrapper {
    fn drop(&mut self) {
        // we will free the qemu main thread lock here
        QEMU_MUTEX_UNLOCK_IOTHREAD.unwrap()();
    }
}

//
// TODO: proper error handling
//
impl PhysicalRead for Wrapper {
    fn phys_read(&mut self, addr: Address, len: Length) -> Result<Vec<u8>> {
        let mut l = len.as_usize() as c_ulonglong;
        let mem = CPU_PHYSICAL_MEMORY_MAP.unwrap()(addr.as_u64(), &mut l, 0);
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
}

impl PhysicalWrite for Wrapper {
    fn phys_write(&mut self, addr: Address, data: &Vec<u8>) -> Result<Length> {
        let mut l = data.len() as c_ulonglong;
        let mem = CPU_PHYSICAL_MEMORY_MAP.unwrap()(addr.as_u64(), &mut l, 1);
        if mem.is_null() {
            Err(Error::new(ErrorKind::Other, "unable to write memory"))
        } else {
            unsafe {
                copy_nonoverlapping(data.as_ptr() as *const c_void, mem, l as usize);
            }
            CPU_PHYSICAL_MEMORY_UNMAP.unwrap()(mem, l, 1, l);
            Ok(Length::from(l as u64))
        }
    }
}
