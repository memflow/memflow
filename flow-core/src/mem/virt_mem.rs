use crate::types::{Address, Length, Page};
use crate::Result;

use std::mem::MaybeUninit;

use dataview::Pod;

pub trait VirtualMemory {
    fn virt_read_raw_iter<'a, VI: VirtualReadIterator<'a>>(&mut self, iter: VI) -> Result<()>;

    fn virt_write_raw_iter<'a, VI: VirtualWriteIterator<'a>>(&mut self, iter: VI) -> Result<()>;

    fn virt_page_info(&mut self, addr: Address) -> Result<Page>;

    // read helpers
    fn virt_read_raw_into(&mut self, addr: Address, out: &mut [u8]) -> Result<()> {
        self.virt_read_raw_iter(Some((addr, out)).into_iter())
    }

    fn virt_read_into<T: Pod + ?Sized>(&mut self, addr: Address, out: &mut T) -> Result<()>
    where
        Self: Sized,
    {
        self.virt_read_raw_into(addr, out.as_bytes_mut())
    }

    fn virt_read_raw(&mut self, addr: Address, len: Length) -> Result<Vec<u8>> {
        let mut buf = vec![0u8; len.as_usize()];
        self.virt_read_raw_into(addr, &mut *buf)?;
        Ok(buf)
    }

    /// # Safety
    ///
    /// this function will overwrite the contents of 'obj' so we can just allocate an unitialized memory section.
    /// this function should only be used with [repr(C)] structs.
    #[allow(clippy::uninit_assumed_init)]
    fn virt_read<T: Pod + Sized>(&mut self, addr: Address) -> Result<T>
    where
        Self: Sized,
    {
        let mut obj: T = unsafe { MaybeUninit::uninit().assume_init() };
        self.virt_read_into(addr, &mut obj)?;
        Ok(obj)
    }

    // write helpers
    fn virt_write_raw(&mut self, addr: Address, data: &[u8]) -> Result<()> {
        self.virt_write_raw_iter(Some((addr, data)).into_iter())
    }

    fn virt_write<T: Pod + ?Sized>(&mut self, addr: Address, data: &T) -> Result<()>
    where
        Self: Sized,
    {
        self.virt_write_raw(addr, data.as_bytes())
    }
}

pub type VirtualReadData<'a> = (Address, &'a mut [u8]);
pub trait VirtualReadIterator<'a>: Iterator<Item = VirtualReadData<'a>> + 'a {}
impl<'a, T: Iterator<Item = VirtualReadData<'a>> + 'a> VirtualReadIterator<'a> for T {}

pub type VirtualWriteData<'a> = (Address, &'a [u8]);
pub trait VirtualWriteIterator<'a>: Iterator<Item = VirtualWriteData<'a>> + 'a {}
impl<'a, T: Iterator<Item = VirtualWriteData<'a>> + 'a> VirtualWriteIterator<'a> for T {}
