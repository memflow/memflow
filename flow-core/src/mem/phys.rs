use super::PageType;
use crate::address::{Address, Length};
use crate::Result;

use std::mem::MaybeUninit;

use dataview::Pod;

// TODO:
// - check endianess here and return an error
// - better would be to convert endianess with word alignment from addr

pub trait AccessPhysicalMemory {
    // system user-defined impls
    fn phys_read_raw_into(
        &mut self,
        addr: Address,
        page_type: PageType,
        out: &mut [u8],
    ) -> Result<()>;

    fn phys_write_raw(&mut self, addr: Address, page_type: PageType, data: &[u8]) -> Result<()>;

    // read helpers
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

    /// # Safety
    ///
    /// this function will overwrite the contents of 'obj' so we can just allocate an unitialized memory section.
    /// this function should only be used with [repr(C)] structs.
    #[allow(clippy::uninit_assumed_init)]
    fn phys_read<T: Pod + Sized>(&mut self, page_type: PageType, addr: Address) -> Result<T> {
        let mut obj: T = unsafe { MaybeUninit::uninit().assume_init() };
        self.phys_read_into(addr, page_type, &mut obj)?;
        Ok(obj)
    }

    // write helpers
    fn phys_write<T: Pod + ?Sized>(
        &mut self,
        addr: Address,
        page_type: PageType,
        data: &T,
    ) -> Result<()> {
        self.phys_write_raw(addr, page_type, data.as_bytes())
    }
}
