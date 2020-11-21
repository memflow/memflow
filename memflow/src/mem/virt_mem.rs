use std::prelude::v1::*;

pub mod virtual_dma;
pub use virtual_dma::VirtualDMA;

use super::VirtualMemoryBatcher;
use crate::architecture::ArchitectureObj;
use crate::error::{Error, PartialError, PartialResult, PartialResultExt, Result};
use crate::types::{Address, Page, PhysicalAddress, Pointer32, Pointer64};

use std::mem::MaybeUninit;

use dataview::Pod;

/// The `VirtualMemory` trait implements access to virtual memory for a specific process
/// and provides a generic way to read and write from/to that processes virtual memory.
///
/// The CPU accesses virtual memory by setting the CR3 register to the appropiate Directory Table Base (DTB)
/// for that process. The ntoskrnl.exe Kernel Process has it's own DTB.
/// Using the DTB it is possible to resolve the physical memory location of a virtual address page.
/// After the address has been resolved the physical memory page can then be read or written to.
///
/// There are 3 methods which are required to be implemented by the provider of this trait.
///
/// # Examples
///
/// Reading from `VirtualMemory`:
/// ```
/// use memflow::types::Address;
/// use memflow::mem::VirtualMemory;
///
/// fn read<T: VirtualMemory>(virt_mem: &mut T, read_addr: Address) {
///     let mut addr = 0u64;
///     virt_mem.virt_read_into(read_addr, &mut addr).unwrap();
///     println!("addr: {:x}", addr);
///     # assert_eq!(addr, 0x00ff_00ff_00ff_00ff);
/// }
/// # use memflow::mem::dummy::DummyMemory;
/// # use memflow::types::size;
/// # let (mut mem, virt_base) = DummyMemory::new_virt(size::mb(4), size::mb(2), &[255, 0, 255, 0, 255, 0, 255, 0]);
/// # read(&mut mem, virt_base);
/// ```
pub trait VirtualMemory
where
    Self: Send,
{
    fn virt_read_raw_list(&mut self, data: &mut [VirtualReadData]) -> PartialResult<()>;

    fn virt_write_raw_list(&mut self, data: &[VirtualWriteData]) -> PartialResult<()>;

    fn virt_page_info(&mut self, addr: Address) -> Result<Page>;

    fn virt_translation_map_range(
        &mut self,
        start: Address,
        end: Address,
    ) -> Vec<(Address, usize, PhysicalAddress)>;

    fn virt_page_map_range(
        &mut self,
        gap_size: usize,
        start: Address,
        end: Address,
    ) -> Vec<(Address, usize)>;

    // read helpers
    fn virt_read_raw_into(&mut self, addr: Address, out: &mut [u8]) -> PartialResult<()> {
        self.virt_read_raw_list(&mut [VirtualReadData(addr, out)])
    }

    fn virt_read_into<T: Pod + ?Sized>(&mut self, addr: Address, out: &mut T) -> PartialResult<()>
    where
        Self: Sized,
    {
        self.virt_read_raw_into(addr, out.as_bytes_mut())
    }

    fn virt_read_raw(&mut self, addr: Address, len: usize) -> PartialResult<Vec<u8>> {
        let mut buf = vec![0u8; len];
        self.virt_read_raw_into(addr, &mut *buf).map_data(|_| buf)
    }

    /// # Safety
    ///
    /// this function will overwrite the contents of 'obj' so we can just allocate an unitialized memory section.
    /// this function should only be used with [repr(C)] structs.
    #[allow(clippy::uninit_assumed_init)]
    fn virt_read<T: Pod + Sized>(&mut self, addr: Address) -> PartialResult<T>
    where
        Self: Sized,
    {
        let mut obj: T = unsafe { MaybeUninit::uninit().assume_init() };
        self.virt_read_into(addr, &mut obj).map_data(|_| obj)
    }

    // write helpers
    fn virt_write_raw(&mut self, addr: Address, data: &[u8]) -> PartialResult<()> {
        self.virt_write_raw_list(&[VirtualWriteData(addr, data)])
    }

    fn virt_write<T: Pod + ?Sized>(&mut self, addr: Address, data: &T) -> PartialResult<()>
    where
        Self: Sized,
    {
        self.virt_write_raw(addr, data.as_bytes())
    }

    // page map helpers
    fn virt_translation_map(&mut self) -> Vec<(Address, usize, PhysicalAddress)> {
        self.virt_translation_map_range(Address::null(), Address::invalid())
    }

    fn virt_page_map(&mut self, gap_size: usize) -> Vec<(Address, usize)> {
        self.virt_page_map_range(gap_size, Address::null(), Address::invalid())
    }

    // specific read helpers
    fn virt_read_addr32(&mut self, addr: Address) -> PartialResult<Address>
    where
        Self: Sized,
    {
        self.virt_read::<u32>(addr).map_data(|d| d.into())
    }

    fn virt_read_addr64(&mut self, addr: Address) -> PartialResult<Address>
    where
        Self: Sized,
    {
        self.virt_read::<u64>(addr).map_data(|d| d.into())
    }

    fn virt_read_addr_arch(
        &mut self,
        arch: ArchitectureObj,
        addr: Address,
    ) -> PartialResult<Address>
    where
        Self: Sized,
    {
        match arch.bits() {
            64 => self.virt_read_addr64(addr),
            32 => self.virt_read_addr32(addr),
            _ => Err(PartialError::Error(Error::InvalidArchitecture)),
        }
    }

    // read pointer wrappers
    fn virt_read_ptr32_into<U: Pod + ?Sized>(
        &mut self,
        ptr: Pointer32<U>,
        out: &mut U,
    ) -> PartialResult<()>
    where
        Self: Sized,
    {
        self.virt_read_into(ptr.address.into(), out)
    }

    fn virt_read_ptr32<U: Pod + Sized>(&mut self, ptr: Pointer32<U>) -> PartialResult<U>
    where
        Self: Sized,
    {
        self.virt_read(ptr.address.into())
    }

    fn virt_read_ptr64_into<U: Pod + ?Sized>(
        &mut self,
        ptr: Pointer64<U>,
        out: &mut U,
    ) -> PartialResult<()>
    where
        Self: Sized,
    {
        self.virt_read_into(ptr.address.into(), out)
    }

    fn virt_read_ptr64<U: Pod + Sized>(&mut self, ptr: Pointer64<U>) -> PartialResult<U>
    where
        Self: Sized,
    {
        self.virt_read(ptr.address.into())
    }

    // TODO: read into slice?
    // TODO: if len is shorter than string -> dynamically double length up to an upper bound
    fn virt_read_cstr(&mut self, addr: Address, len: usize) -> PartialResult<String> {
        let mut buf = vec![0; len];
        self.virt_read_raw_into(addr, &mut buf).data_part()?;
        if let Some((n, _)) = buf.iter().enumerate().find(|(_, c)| **c == 0_u8) {
            buf.truncate(n);
        }
        Ok(String::from_utf8_lossy(&buf).to_string())
    }

    fn virt_batcher(&mut self) -> VirtualMemoryBatcher<Self>
    where
        Self: Sized,
    {
        VirtualMemoryBatcher::new(self)
    }
}

// forward impls
impl<T: VirtualMemory + ?Sized, P: std::ops::DerefMut<Target = T> + Send> VirtualMemory for P {
    #[inline]
    fn virt_read_raw_list(&mut self, data: &mut [VirtualReadData]) -> PartialResult<()> {
        (**self).virt_read_raw_list(data)
    }

    #[inline]
    fn virt_write_raw_list(&mut self, data: &[VirtualWriteData]) -> PartialResult<()> {
        (**self).virt_write_raw_list(data)
    }

    #[inline]
    fn virt_page_info(&mut self, addr: Address) -> Result<Page> {
        (**self).virt_page_info(addr)
    }

    #[inline]
    fn virt_translation_map_range(
        &mut self,
        start: Address,
        end: Address,
    ) -> Vec<(Address, usize, PhysicalAddress)> {
        (**self).virt_translation_map_range(start, end)
    }

    #[inline]
    fn virt_page_map_range(
        &mut self,
        gap_size: usize,
        start: Address,
        end: Address,
    ) -> Vec<(Address, usize)> {
        (**self).virt_page_map_range(gap_size, start, end)
    }
}

// iterator helpers
#[repr(C)]
pub struct VirtualReadData<'a>(pub Address, pub &'a mut [u8]);
pub trait VirtualReadIterator<'a>: Iterator<Item = VirtualReadData<'a>> + 'a {}
impl<'a, T: Iterator<Item = VirtualReadData<'a>> + 'a> VirtualReadIterator<'a> for T {}

impl<'a> From<VirtualReadData<'a>> for (Address, &'a mut [u8]) {
    fn from(VirtualReadData(a, b): VirtualReadData<'a>) -> Self {
        (a, b)
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct VirtualWriteData<'a>(pub Address, pub &'a [u8]);
pub trait VirtualWriteIterator<'a>: Iterator<Item = VirtualWriteData<'a>> + 'a {}
impl<'a, T: Iterator<Item = VirtualWriteData<'a>> + 'a> VirtualWriteIterator<'a> for T {}

impl<'a> From<VirtualWriteData<'a>> for (Address, &'a [u8]) {
    fn from(VirtualWriteData(a, b): VirtualWriteData<'a>) -> Self {
        (a, b)
    }
}
