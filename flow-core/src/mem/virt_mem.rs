use std::prelude::v1::*;

pub mod virt_from_phys;
pub use virt_from_phys::VirtualFromPhysical;

use super::VirtualMemoryBatcher;
use crate::error::{Error, Result};
use crate::types::{Address, Page, Pointer32, Pointer64};

#[cfg(feature = "std")]
use std::ffi::CString;
use std::mem::MaybeUninit;

use dataview::Pod;

/**
The `VirtualMemory` trait implements access to virtual memory for a specific process
and provides a generic way to read and write from/to that processes virtual memory.

The CPU accesses virtual memory by setting the CR3 register to the appropiate Directory Table Base (DTB)
for that process. The ntoskrnl.exe Kernel Process has it's own DTB.
Using the DTB it is possible to resolve the physical memory location of a virtual address page.
After the address has been resolved the physical memory page can then be read or written to.

There are 3 methods which are required to be implemented by the provider of this trait.

# Examples

Reading from `VirtualMemory`:
```
use flow_core::types::Address;
use flow_core::mem::VirtualMemory;

fn read<T: VirtualMemory>(virt_mem: &mut T) {
    let mut addr = 0u64;
    virt_mem.virt_read_into(Address::from(0x1000), &mut addr).unwrap();
    println!("addr: {:x}", addr);
}
```
*/
pub trait VirtualMemory {
    fn virt_read_raw_iter<'a, VI: VirtualReadIterator<'a>>(&mut self, iter: VI) -> Result<()>;

    fn virt_write_raw_iter<'a, VI: VirtualWriteIterator<'a>>(&mut self, iter: VI) -> Result<()>;

    fn virt_page_info(&mut self, addr: Address) -> Result<Page>;

    fn virt_page_map(&mut self, gap_size: usize) -> Vec<(Address, usize)>;

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

    fn virt_read_raw(&mut self, addr: Address, len: usize) -> Result<Vec<u8>> {
        let mut buf = vec![0u8; len];
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

    // specific read helpers
    fn virt_read_addr32(&mut self, addr: Address) -> Result<Address>
    where
        Self: Sized,
    {
        Ok(self.virt_read::<u32>(addr)?.into())
    }

    fn virt_read_addr64(&mut self, addr: Address) -> Result<Address>
    where
        Self: Sized,
    {
        Ok(self.virt_read::<u64>(addr)?.into())
    }

    // read pointer wrappers
    fn virt_read_ptr32_into<U: Pod + ?Sized>(
        &mut self,
        ptr: Pointer32<U>,
        out: &mut U,
    ) -> Result<()>
    where
        Self: Sized,
    {
        self.virt_read_into(ptr.address.into(), out)
    }

    fn virt_read_ptr32<U: Pod + Sized>(&mut self, ptr: Pointer32<U>) -> Result<U>
    where
        Self: Sized,
    {
        self.virt_read(ptr.address.into())
    }

    fn virt_read_ptr64_into<U: Pod + ?Sized>(
        &mut self,
        ptr: Pointer64<U>,
        out: &mut U,
    ) -> Result<()>
    where
        Self: Sized,
    {
        self.virt_read_into(ptr.address.into(), out)
    }

    fn virt_read_ptr64<U: Pod + Sized>(&mut self, ptr: Pointer64<U>) -> Result<U>
    where
        Self: Sized,
    {
        self.virt_read(ptr.address.into())
    }

    // TODO: read into slice?
    // TODO: if len is shorter than string truncate it!
    #[cfg(feature = "std")]
    fn virt_read_cstr(&mut self, addr: Address, len: usize) -> Result<String> {
        let mut buf = vec![0; len];
        self.virt_read_raw_into(addr, &mut buf)?;
        if let Some((n, _)) = buf.iter().enumerate().find(|(_, c)| **c == 0_u8) {
            buf.truncate(n);
        }
        let v = CString::new(buf).map_err(|_| Error::Encoding)?;
        Ok(String::from(v.to_string_lossy()))
    }

    fn virt_batcher(&mut self) -> VirtualMemoryBatcher<Self>
    where
        Self: Sized,
    {
        VirtualMemoryBatcher::new(self)
    }
}

// forward impls
impl<'a, T: VirtualMemory> VirtualMemory for &'a mut T {
    fn virt_read_raw_iter<'b, VI: VirtualReadIterator<'b>>(&mut self, iter: VI) -> Result<()> {
        (*self).virt_read_raw_iter(iter)
    }

    fn virt_write_raw_iter<'b, VI: VirtualWriteIterator<'b>>(&mut self, iter: VI) -> Result<()> {
        (*self).virt_write_raw_iter(iter)
    }

    fn virt_page_info(&mut self, addr: Address) -> Result<Page> {
        (*self).virt_page_info(addr)
    }

    fn virt_page_map(&mut self, gap_size: usize) -> Vec<(Address, usize)> {
        (*self).virt_page_map(gap_size)
    }
}

// iterator helpers
pub type VirtualReadData<'a> = (Address, &'a mut [u8]);
pub trait VirtualReadIterator<'a>: Iterator<Item = VirtualReadData<'a>> + 'a {}
impl<'a, T: Iterator<Item = VirtualReadData<'a>> + 'a> VirtualReadIterator<'a> for T {}

pub type VirtualWriteData<'a> = (Address, &'a [u8]);
pub trait VirtualWriteIterator<'a>: Iterator<Item = VirtualWriteData<'a>> + 'a {}
impl<'a, T: Iterator<Item = VirtualWriteData<'a>> + 'a> VirtualWriteIterator<'a> for T {}
