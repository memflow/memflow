use crate::types::{Length, PhysicalAddress};
use crate::Result;

use std::mem::MaybeUninit;

use dataview::Pod;

// TODO:
// - check endianess here and return an error
// - better would be to convert endianess with word alignment from addr

/**
The `AccessPhysicalMemory` trait is implemented by memory backends
and provides a generic way to read and write from/to physical memory.

All addresses are of the type [`PhysicalAddress`](../types/physical_address/index.html)
and can contain additional information about the page the address resides in.
This information is usually only needed when implementing caches.

There are only 2 methods which are required to be implemented by the provider of this trait.

# Examples

Reading physical memory with `AccessPhysicalMemory`:
```
use flow_core::mem::AccessPhysicalMemory;
use flow_core::types::Address;

fn test<T: AccessPhysicalMemory>(mem: &mut T) {
    let mut value = 0u64;
    mem.phys_read_into(Address::from(0x1000).into(), &mut value);
}
```

Implementing `AccessPhysicalMemory` for a memory backend:
```
use std::vec::Vec;

use flow_core::mem::{AccessPhysicalMemory, PhysicalReadIterator, PhysicalWriteIterator};
use flow_core::types::{PhysicalAddress, ToDo, Done};
use flow_core::error::Result;

pub struct MemoryBackend {
    mem: Box<[u8]>,
}

impl AccessPhysicalMemory for MemoryBackend {
    fn phys_read_raw_iter<'a, PI: PhysicalReadIterator<'a>>(
        &'a mut self,
        iter: PI
    ) -> Result<()> {
        iter.for_each(|(addr, out)| out.copy_from_slice(&self.mem[addr.as_usize()..(addr.as_usize() + out.len())]));
        Ok(())
    }

    fn phys_write_raw_iter<'a, PI: PhysicalWriteIterator<'a>>(
        &'a mut self,
        iter: PI
    ) -> Result<()> {
        iter.for_each(|(addr, data)| self.mem[addr.as_usize()..(addr.as_usize() + data.len())].copy_from_slice(data));
        Ok(())
    }
}
```
*/
pub trait AccessPhysicalMemory {
    // required to be implemented
    fn phys_read_raw_iter<'a, PI: PhysicalReadIterator<'a>>(&'a mut self, iter: PI) -> Result<()>;

    fn phys_write_raw_iter<'a, PI: PhysicalWriteIterator<'a>>(&'a mut self, iter: PI)
        -> Result<()>;

    // read helpers
    fn phys_read_raw_into(&mut self, addr: PhysicalAddress, out: &mut [u8]) -> Result<()> {
        self.phys_read_raw_iter(Some((addr, out)).into_iter())
    }

    fn phys_read_into<T: Pod + ?Sized>(&mut self, addr: PhysicalAddress, out: &mut T) -> Result<()>
    where
        Self: Sized,
    {
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
    fn phys_read<T: Pod + Sized>(&mut self, addr: PhysicalAddress) -> Result<T>
    where
        Self: Sized,
    {
        let mut obj: T = unsafe { MaybeUninit::uninit().assume_init() };
        self.phys_read_into(addr, &mut obj)?;
        Ok(obj)
    }

    // write helpers
    fn phys_write_raw(&mut self, addr: PhysicalAddress, data: &[u8]) -> Result<()> {
        self.phys_write_raw_iter(Some((addr, data)).into_iter())
    }

    fn phys_write<T: Pod + ?Sized>(&mut self, addr: PhysicalAddress, data: &T) -> Result<()>
    where
        Self: Sized,
    {
        self.phys_write_raw(addr, data.as_bytes())
    }
}

pub type PhysicalReadData<'a> = (PhysicalAddress, &'a mut [u8]);
pub trait PhysicalReadIterator<'a>: Iterator<Item = PhysicalReadData<'a>> + 'a {}
impl<'a, T: Iterator<Item = PhysicalReadData<'a>> + 'a> PhysicalReadIterator<'a> for T {}

pub type PhysicalWriteData<'a> = (PhysicalAddress, &'a [u8]);
pub trait PhysicalWriteIterator<'a>: Iterator<Item = PhysicalWriteData<'a>> + 'a {}
impl<'a, T: Iterator<Item = PhysicalWriteData<'a>> + 'a> PhysicalWriteIterator<'a> for T {}
