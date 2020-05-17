use super::AccessVirtualMemory;
use crate::architecture::Architecture;
use crate::error::Error;
use crate::types::{Address, Length, Pointer32, Pointer64};
use crate::Result;

use std::ffi::CString;

use dataview::Pod;

pub struct VirtualMemoryContext<'a, T: AccessVirtualMemory + ?Sized> {
    mem: &'a mut T,
    sys_arch: Architecture,
    proc_arch: Architecture,
    dtb: Address,
}

impl<'a, T: AccessVirtualMemory + ?Sized> VirtualMemoryContext<'a, T> {
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

    pub fn virt_read_raw(&mut self, addr: Address, len: Length) -> Result<Vec<u8>> {
        self.mem.virt_read_raw(self.sys_arch, self.dtb, addr, len)
    }

    pub fn virt_write_raw(&mut self, addr: Address, data: &[u8]) -> Result<()> {
        self.mem.virt_write_raw(self.sys_arch, self.dtb, addr, data)
    }
}

// sized impl
impl<'a, T: AccessVirtualMemory + Sized> VirtualMemoryContext<'a, T> {
    pub fn virt_read_into<U: Pod + ?Sized>(&mut self, addr: Address, out: &mut U) -> Result<()> {
        self.mem.virt_read_into(self.sys_arch, self.dtb, addr, out)
    }

    pub fn virt_read<U: Pod + Sized>(&mut self, addr: Address) -> Result<U> {
        self.mem.virt_read(self.sys_arch, self.dtb, addr)
    }

    pub fn virt_write<U: Pod + ?Sized>(&mut self, addr: Address, data: &U) -> Result<()> {
        self.mem.virt_write(self.sys_arch, self.dtb, addr, data)
    }

    // read address wrappers
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
            _ => Err(Error::new("invalid instruction set")),
        }
    }

    // read pointer wrappers
    pub fn virt_read_ptr32_into<U: Pod + ?Sized>(
        &mut self,
        ptr: Pointer32<U>,
        out: &mut U,
    ) -> Result<()> {
        self.virt_read_into(ptr.address.into(), out)
    }

    pub fn virt_read_ptr32<U: Pod + Sized>(&mut self, ptr: Pointer32<U>) -> Result<U> {
        self.virt_read(ptr.address.into())
    }

    pub fn virt_read_ptr64_into<U: Pod + ?Sized>(
        &mut self,
        ptr: Pointer64<U>,
        out: &mut U,
    ) -> Result<()> {
        self.virt_read_into(ptr.address.into(), out)
    }

    pub fn virt_read_ptr64<U: Pod + Sized>(&mut self, ptr: Pointer64<U>) -> Result<U> {
        self.virt_read(ptr.address.into())
    }

    // TODO: read into slice?
    // TODO: if len is shorter than string truncate it!
    pub fn virt_read_cstr(&mut self, addr: Address, len: Length) -> Result<String> {
        let mut buf = vec![0; len.as_usize()];
        self.virt_read_raw_into(addr, &mut buf)?;
        if let Some((n, _)) = buf.iter().enumerate().find(|(_, c)| **c == 0_u8) {
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
