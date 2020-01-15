use std::ffi::CString;
use std::os::raw::{c_ulonglong, c_void};

use std::ptr::copy_nonoverlapping;

use flow_core::*;

use crate::native::*;

pub struct Memory;

impl Memory {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self {}
    }
}

impl PhysicalMemoryTrait for Memory {
    fn phys_read(&mut self, addr: Address, out: &mut [u8]) -> Result<()> {
        Wrapper::new().phys_read(addr, out)
    }

    fn phys_write(&mut self, addr: Address, data: &[u8]) -> Result<()> {
        Wrapper::new().phys_write(addr, data)
    }
}

impl VirtualMemoryTrait for Memory {
    fn virt_read(
        &mut self,
        arch: Architecture,
        dtb: Address,
        addr: Address,
        out: &mut [u8],
    ) -> Result<()> {
        VatImpl::new(&mut Wrapper::new()).virt_read(arch, dtb, addr, out)
    }

    fn virt_write(
        &mut self,
        arch: Architecture,
        dtb: Address,
        addr: Address,
        data: &[u8],
    ) -> Result<()> {
        VatImpl::new(&mut Wrapper::new()).virt_write(arch, dtb, addr, data)
    }
}

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
impl PhysicalMemoryTrait for Wrapper {
    fn phys_read(&mut self, addr: Address, out: &mut [u8]) -> Result<()> {
        let mut l = out.len() as c_ulonglong;
        let mem = CPU_PHYSICAL_MEMORY_MAP.unwrap()(addr.as_u64(), &mut l, 0);
        if mem.is_null() {
            Err(Error::new("unable to read memory"))
        } else {
            unsafe {
                copy_nonoverlapping(mem, out.as_mut_ptr() as *mut c_void, l as usize);
            }
            CPU_PHYSICAL_MEMORY_UNMAP.unwrap()(mem, l, 0, l);
            Ok(())
        }
    }

    fn phys_write(&mut self, addr: Address, data: &[u8]) -> Result<()> {
        let mut l = data.len() as c_ulonglong;
        let mem = CPU_PHYSICAL_MEMORY_MAP.unwrap()(addr.as_u64(), &mut l, 1);
        if mem.is_null() {
            Err(Error::new("unable to write memory"))
        } else {
            unsafe {
                copy_nonoverlapping(data.as_ptr() as *const c_void, mem, l as usize);
            }
            CPU_PHYSICAL_MEMORY_UNMAP.unwrap()(mem, l, 1, l);
            Ok(())
        }
    }
}
