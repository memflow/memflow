use crate::address::{Length, PhysicalAddress};
use crate::Result;

use std::mem::MaybeUninit;

use dataview::Pod;

// TODO:
// - check endianess here and return an error
// - better would be to convert endianess with word alignment from addr

pub trait AccessPhysicalMemory {
    // system user-defined impls
    fn phys_read_raw_into(&mut self, addr: PhysicalAddress, out: &mut [u8]) -> Result<()>;

    fn phys_write_raw(&mut self, addr: PhysicalAddress, data: &[u8]) -> Result<()>;

    // read helpers
    fn phys_read_into<T: Pod + ?Sized>(
        &mut self,
        addr: PhysicalAddress,
        out: &mut T,
    ) -> Result<()> {
        self.phys_read_raw_into(addr, out.as_bytes_mut())
    }

    fn phys_read_raw(&mut self, addr: PhysicalAddress, len: Length) -> Result<Vec<u8>> {
        let mut buf = vec![0u8; len.as_usize()];
        self.phys_read_raw_into(addr, &mut *buf)?;
        Ok(buf)
    }

    /// # Safety
    ///
    /// this function will overwrite the contents of 'obj' so we can just allocate an unitialized memory section.
    /// this function should only be used with [repr(C)] structs.
    #[allow(clippy::uninit_assumed_init)]
    fn phys_read<T: Pod + Sized>(&mut self, addr: PhysicalAddress) -> Result<T> {
        let mut obj: T = unsafe { MaybeUninit::uninit().assume_init() };
        self.phys_read_into(addr, &mut obj)?;
        Ok(obj)
    }

    // write helpers
    fn phys_write<T: Pod + ?Sized>(&mut self, addr: PhysicalAddress, data: &T) -> Result<()> {
        self.phys_write_raw(addr, data.as_bytes())
    }
}
