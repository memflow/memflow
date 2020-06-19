use std::prelude::v1::*;

use super::PhysicalMemoryBatcher;
use crate::types::PhysicalAddress;
use crate::Result;

use std::mem::MaybeUninit;

use dataview::Pod;

// TODO:
// - check endianess here and return an error
// - better would be to convert endianess with word alignment from addr

/**
The `PhysicalMemory` trait is implemented by memory backends
and provides a generic way to read and write from/to physical memory.

All addresses are of the type [`PhysicalAddress`](../types/physical_address/index.html)
and can contain additional information about the page the address resides in.
This information is usually only needed when implementing caches.

There are only 2 methods which are required to be implemented by the provider of this trait.

# Examples

Implementing `PhysicalMemory` for a memory backend:
```
use std::vec::Vec;

use flow_core::mem::{PhysicalMemory, PhysicalReadIterator, PhysicalWriteIterator};
use flow_core::types::{PhysicalAddress, ToDo, Done};
use flow_core::error::Result;

pub struct MemoryBackend {
    mem: Box<[u8]>,
}

impl PhysicalMemory for MemoryBackend {
    fn phys_read_iter<'a, PI: PhysicalReadIterator<'a>>(
        &'a mut self,
        iter: PI
    ) -> Result<()> {
        iter.for_each(|(addr, out)| out.copy_from_slice(&self.mem[addr.as_usize()..(addr.as_usize() + out.len())]));
        Ok(())
    }

    fn phys_write_iter<'a, PI: PhysicalWriteIterator<'a>>(
        &'a mut self,
        iter: PI
    ) -> Result<()> {
        iter.for_each(|(addr, data)| self.mem[addr.as_usize()..(addr.as_usize() + data.len())].copy_from_slice(data));
        Ok(())
    }
}
```

Reading from `PhysicalMemory`:
```
use flow_core::types::Address;
use flow_core::mem::PhysicalMemory;

fn read<T: PhysicalMemory>(mem: &mut T) {
    let mut addr = 0u64;
    mem.phys_read_into(Address::from(0x1000).into(), &mut addr).unwrap();
    println!("addr: {:x}", addr);
}
```
*/
pub trait PhysicalMemory {
    fn phys_read_iter<'a, PI: PhysicalReadIterator<'a>>(&'a mut self, iter: PI) -> Result<()>;
    fn phys_write_iter<'a, PI: PhysicalWriteIterator<'a>>(&'a mut self, iter: PI) -> Result<()>;

    // read helpers
    fn phys_read_raw_into(&mut self, addr: PhysicalAddress, out: &mut [u8]) -> Result<()> {
        self.phys_read_iter(Some((addr, out)).into_iter())
    }

    fn phys_read_into<T: Pod + ?Sized>(&mut self, addr: PhysicalAddress, out: &mut T) -> Result<()>
    where
        Self: Sized,
    {
        self.phys_read_raw_into(addr, out.as_bytes_mut())
    }

    fn phys_read_raw(&mut self, addr: PhysicalAddress, len: usize) -> Result<Vec<u8>> {
        let mut buf = vec![0u8; len];
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
        self.phys_write_iter(Some((addr, data)).into_iter())
    }

    fn phys_write<T: Pod + ?Sized>(&mut self, addr: PhysicalAddress, data: &T) -> Result<()>
    where
        Self: Sized,
    {
        self.phys_write_raw(addr, data.as_bytes())
    }

    fn phys_batcher(&mut self) -> PhysicalMemoryBatcher<Self>
    where
        Self: Sized,
    {
        PhysicalMemoryBatcher::new(self)
    }
}

// forward impls
impl<'a, T: PhysicalMemory> PhysicalMemory for &'a mut T {
    fn phys_read_iter<'b, PI: PhysicalReadIterator<'b>>(&'b mut self, iter: PI) -> Result<()> {
        (*self).phys_read_iter(iter)
    }

    fn phys_write_iter<'b, PI: PhysicalWriteIterator<'b>>(&'b mut self, iter: PI) -> Result<()> {
        (*self).phys_write_iter(iter)
    }
}

// iterator helpers
pub type PhysicalReadData<'a> = (PhysicalAddress, &'a mut [u8]);
pub trait PhysicalReadIterator<'a>: Iterator<Item = PhysicalReadData<'a>> + 'a {}
impl<'a, T: Iterator<Item = PhysicalReadData<'a>> + 'a> PhysicalReadIterator<'a> for T {}

pub type PhysicalWriteData<'a> = (PhysicalAddress, &'a [u8]);
pub trait PhysicalWriteIterator<'a>: Iterator<Item = PhysicalWriteData<'a>> + 'a {}
impl<'a, T: Iterator<Item = PhysicalWriteData<'a>> + 'a> PhysicalWriteIterator<'a> for T {}
