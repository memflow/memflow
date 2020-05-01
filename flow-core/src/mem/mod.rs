pub mod cache;
pub mod vat;

use crate::address::{Address, Length};
use crate::arch::Architecture;
use crate::error::Error;
use crate::Result;

use std::ffi::CString;
use std::mem::MaybeUninit;

use dataview::Pod;

bitflags! {
    pub struct PageType: u8 {
        const NONE = 0;
        const PAGE_TABLE = 1;
        const WRITEABLE = 2;
        const READ_ONLY = 3;
    }
}

impl PageType {
    pub fn from_writeable_bit(writeable: bool) -> Self {
        match writeable {
            true => PageType::WRITEABLE,
            false => PageType::READ_ONLY,
        }
    }
}

pub trait MemCache {
    fn cached_read<F: FnMut(Address, &mut [u8]) -> Result<()>>(
        &mut self,
        start: Address,
        page_type: PageType,
        out: &mut [u8],
        read_fn: F,
    ) -> Result<usize>;
    fn cache_page(&mut self, addr: Address, page_type: PageType, src: &[u8]);
    fn invalidate_pages(&mut self, addr: Address, page_type: PageType, src: &[u8]);
}

// TODO:
// - check endianess here and return an error
// - better would be to convert endianess with word alignment from addr

// generic traits
pub trait AccessPhysicalMemory {
    // read
    fn phys_read_raw_into(
        &mut self,
        addr: Address,
        page_type: PageType,
        out: &mut [u8],
    ) -> Result<()>;

    fn phys_read_into<T: Pod + ?Sized>(
        &mut self,
        addr: Address,
        page_type: PageType,
        out: &mut T,
    ) -> Result<()> {
        self.phys_read_raw_into(addr, page_type, out.as_bytes_mut())
    }

    fn phys_read_raw(
        &mut self,
        addr: Address,
        page_type: PageType,
        len: Length,
    ) -> Result<Vec<u8>> {
        let mut buf = vec![0u8; len.as_usize()];
        self.phys_read_raw_into(addr, page_type, &mut *buf)?;
        Ok(buf)
    }

    fn phys_read<T: Pod + Sized>(&mut self, page_type: PageType, addr: Address) -> Result<T> {
        let mut obj: T = unsafe { MaybeUninit::uninit().assume_init() };
        self.phys_read_into(addr, page_type, &mut obj)?;
        Ok(obj)
    }

    // write
    fn phys_write_raw(&mut self, addr: Address, page_type: PageType, data: &[u8]) -> Result<()>;

    fn phys_write<T: Pod + ?Sized>(
        &mut self,
        addr: Address,
        page_type: PageType,
        data: &T,
    ) -> Result<()> {
        self.phys_write_raw(addr, page_type, data.as_bytes())
    }
}

pub trait AccessVirtualMemory {
    // read
    fn virt_read_raw_into(
        &mut self,
        arch: Architecture,
        dtb: Address,
        addr: Address,
        out: &mut [u8],
    ) -> Result<()>;

    fn virt_read_into<T: Pod + ?Sized>(
        &mut self,
        arch: Architecture,
        dtb: Address,
        addr: Address,
        out: &mut T,
    ) -> Result<()> {
        self.virt_read_raw_into(arch, dtb, addr, out.as_bytes_mut())
    }

    fn virt_read_raw(
        &mut self,
        arch: Architecture,
        dtb: Address,
        addr: Address,
        len: Length,
    ) -> Result<Vec<u8>> {
        let mut buf = vec![0u8; len.as_usize()];
        self.virt_read_raw_into(arch, dtb, addr, &mut *buf)?;
        Ok(buf)
    }

    fn virt_read<T: Pod + Sized>(
        &mut self,
        arch: Architecture,
        dtb: Address,
        addr: Address,
    ) -> Result<T> {
        let mut obj: T = unsafe { MaybeUninit::uninit().assume_init() };
        self.virt_read_into(arch, dtb, addr, &mut obj)?;
        Ok(obj)
    }

    // write
    fn virt_write_raw(
        &mut self,
        arch: Architecture,
        dtb: Address,
        addr: Address,
        data: &[u8],
    ) -> Result<()>;

    fn virt_write<T: Pod + ?Sized>(
        &mut self,
        arch: Architecture,
        dtb: Address,
        addr: Address,
        data: &T,
    ) -> Result<()> {
        self.virt_write_raw(arch, dtb, addr, data.as_bytes())
    }
}

pub struct VirtualMemoryContext<'a, T: AccessVirtualMemory> {
    mem: &'a mut T,
    sys_arch: Architecture,
    proc_arch: Architecture,
    dtb: Address,
}

impl<'a, T: AccessVirtualMemory> VirtualMemoryContext<'a, T> {
    pub fn with(mem: &'a mut T, sys_arch: Architecture, dtb: Address) -> Self {
        Self {
            mem,
            sys_arch,
            proc_arch: sys_arch,
            dtb,
        }
    }

    pub fn with_proc_arch(
        mem: &'a mut T,
        sys_arch: Architecture,
        proc_arch: Architecture,
        dtb: Address,
    ) -> Self {
        Self {
            mem,
            sys_arch,
            proc_arch,
            dtb,
        }
    }

    pub fn sys_arch(&self) -> Architecture {
        self.sys_arch
    }

    pub fn proc_arch(&self) -> Architecture {
        self.proc_arch
    }

    pub fn dtb(&self) -> Address {
        self.dtb
    }

    // self.mem wrappers
    pub fn virt_read_raw_into(&mut self, addr: Address, out: &mut [u8]) -> Result<()> {
        self.mem
            .virt_read_raw_into(self.sys_arch, self.dtb, addr, out)
    }

    pub fn virt_read_into<U: Pod + ?Sized>(&mut self, addr: Address, out: &mut U) -> Result<()> {
        self.mem.virt_read_into(self.sys_arch, self.dtb, addr, out)
    }

    pub fn virt_read_raw(&mut self, addr: Address, len: Length) -> Result<Vec<u8>> {
        self.mem.virt_read_raw(self.sys_arch, self.dtb, addr, len)
    }

    pub fn virt_read<U: Pod + Sized>(&mut self, addr: Address) -> Result<U> {
        self.mem.virt_read(self.sys_arch, self.dtb, addr)
    }

    pub fn virt_write_raw(&mut self, addr: Address, data: &[u8]) -> Result<()> {
        self.mem.virt_write_raw(self.sys_arch, self.dtb, addr, data)
    }

    pub fn virt_write<U: Pod + ?Sized>(&mut self, addr: Address, data: &U) -> Result<()> {
        self.mem.virt_write(self.sys_arch, self.dtb, addr, data)
    }

    // custom read wrappers
    pub fn virt_read_addr32(&mut self, addr: Address) -> Result<Address> {
        let mut res = 0u32;
        self.virt_read_into(addr, &mut res)?;
        Ok(Address::from(res))
    }

    pub fn virt_read_addr64(&mut self, addr: Address) -> Result<Address> {
        let mut res = 0u64;
        self.virt_read_into(addr, &mut res)?;
        Ok(Address::from(res))
    }

    pub fn virt_read_addr(&mut self, addr: Address) -> Result<Address> {
        match self.proc_arch.bits() {
            64 => self.virt_read_addr64(addr),
            32 => self.virt_read_addr32(addr),
            _ => return Err(Error::new("invalid instruction set")),
        }
    }

    // TODO: read into slice?
    // TODO: if len is shorter than string truncate it!
    pub fn virt_read_cstr(&mut self, addr: Address, len: Length) -> Result<String> {
        let mut buf = vec![0; len.as_usize()];
        self.virt_read_raw_into(addr, &mut buf)?;
        if let Some((n, _)) = buf.iter().enumerate().filter(|(_, c)| **c == 0_u8).nth(0) {
            buf.truncate(n);
        }
        let v = CString::new(buf)?;
        Ok(String::from(v.to_string_lossy()))
    }

    // TODO: read into slice?
    // TODO: if len is shorter than string truncate it!
    pub fn virt_read_cstr_ptr(&mut self, addr: Address) -> Result<String> {
        let ptr = self.virt_read_addr(addr)?;
        self.virt_read_cstr(ptr, Length::from_kb(2))
    }

    pub fn virt_read_addr_chain(
        &mut self,
        base_addr: Address,
        offsets: Vec<Length>,
    ) -> Result<Address> {
        offsets
            .iter()
            .try_fold(base_addr, |c, &a| self.virt_read_addr(c + a))
    }
}
