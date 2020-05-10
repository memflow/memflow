use crate::arch::Architecture;
use crate::types::{Address, Length, Page};
use crate::Result;

use std::mem::MaybeUninit;

use dataview::Pod;

pub trait AccessVirtualMemory {
    // system user-defined impls
    fn virt_read_raw_into(
        &mut self,
        arch: Architecture,
        dtb: Address,
        addr: Address,
        out: &mut [u8],
    ) -> Result<()>;

    fn virt_write_raw(
        &mut self,
        arch: Architecture,
        dtb: Address,
        addr: Address,
        data: &[u8],
    ) -> Result<()>;

    fn virt_page_info(&mut self, arch: Architecture, dtb: Address, addr: Address) -> Result<Page>;

    // read helpers
    fn virt_read_into<T: Pod + ?Sized>(
        &mut self,
        arch: Architecture,
        dtb: Address,
        addr: Address,
        out: &mut T,
    ) -> Result<()>
    where
        Self: Sized,
    {
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

    /// # Safety
    ///
    /// this function will overwrite the contents of 'obj' so we can just allocate an unitialized memory section.
    /// this function should only be used with [repr(C)] structs.
    #[allow(clippy::uninit_assumed_init)]
    fn virt_read<T: Pod + Sized>(
        &mut self,
        arch: Architecture,
        dtb: Address,
        addr: Address,
    ) -> Result<T>
    where
        Self: Sized,
    {
        let mut obj: T = unsafe { MaybeUninit::uninit().assume_init() };
        self.virt_read_into(arch, dtb, addr, &mut obj)?;
        Ok(obj)
    }

    // write helpers
    fn virt_write<T: Pod + ?Sized>(
        &mut self,
        arch: Architecture,
        dtb: Address,
        addr: Address,
        data: &T,
    ) -> Result<()>
    where
        Self: Sized,
    {
        self.virt_write_raw(arch, dtb, addr, data.as_bytes())
    }
}
