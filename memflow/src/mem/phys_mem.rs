use std::prelude::v1::*;

use super::PhysicalMemoryBatcher;
use crate::error::Result;
use crate::types::PhysicalAddress;

use std::mem::MaybeUninit;

use dataview::Pod;

// TODO:
// - check endianess here and return an error
// - better would be to convert endianess with word alignment from addr

/// The `PhysicalMemory` trait is implemented by memory backends
/// and provides a generic way to read and write from/to physical memory.
///
/// All addresses are of the type [`PhysicalAddress`](../types/physical_address/index.html)
/// and can contain additional information about the page the address resides in.
/// This information is usually only needed when implementing caches.
///
/// There are only 2 methods which are required to be implemented by the provider of this trait.
///
/// # Examples
///
/// Implementing `PhysicalMemory` for a memory backend:
/// ```
/// use std::vec::Vec;
///
/// use memflow::mem::{
///     PhysicalMemory,
///     PhysicalReadData,
///     PhysicalWriteData,
///     PhysicalMemoryMetadata
/// };
///
/// use memflow::types::PhysicalAddress;
/// use memflow::error::Result;
///
/// pub struct MemoryBackend {
///     mem: Box<[u8]>,
/// }
///
/// impl PhysicalMemory for MemoryBackend {
///     fn phys_read_raw_list(
///         &mut self,
///         data: &mut [PhysicalReadData]
///     ) -> Result<()> {
///         data
///             .iter_mut()
///             .for_each(|PhysicalReadData(addr, out)| out
///                 .copy_from_slice(&self.mem[addr.as_usize()..(addr.as_usize() + out.len())])
///             );
///         Ok(())
///     }
///
///     fn phys_write_raw_list(
///         &mut self,
///         data: &[PhysicalWriteData]
///     ) -> Result<()> {
///         data
///             .iter()
///             .for_each(|PhysicalWriteData(addr, data)| self
///                 .mem[addr.as_usize()..(addr.as_usize() + data.len())].copy_from_slice(data)
///             );
///         Ok(())
///     }
///
///     fn metadata(&self) -> PhysicalMemoryMetadata {
///         PhysicalMemoryMetadata {
///             size: self.mem.len(),
///             readonly: false
///         }
///     }
/// }
/// ```
///
/// Reading from `PhysicalMemory`:
/// ```
/// use memflow::types::Address;
/// use memflow::mem::PhysicalMemory;
///
/// fn read<T: PhysicalMemory>(mem: &mut T) {
///     let mut addr = 0u64;
///     mem.phys_read_into(Address::from(0x1000).into(), &mut addr).unwrap();
///     println!("addr: {:x}", addr);
/// }
///
/// # use memflow::mem::dummy::DummyMemory;
/// # use memflow::types::size;
/// # read(&mut DummyMemory::new(size::mb(4)));
/// ```
pub trait PhysicalMemory
where
    Self: Send,
{
    fn phys_read_raw_list(&mut self, data: &mut [PhysicalReadData]) -> Result<()>;
    fn phys_write_raw_list(&mut self, data: &[PhysicalWriteData]) -> Result<()>;

    /// Retrieve metadata about the physical memory
    ///
    /// This function will return metadata about the underlying physical memory object, currently
    /// including address space size and read-only status.
    ///
    /// # Examples
    ///
    /// ```
    /// use memflow::types::size;
    /// use memflow::mem::PhysicalMemory;
    /// # let mem = memflow::mem::dummy::DummyMemory::new(size::mb(16));
    ///
    /// let metadata = mem.metadata();
    ///
    /// assert_eq!(metadata.size, size::mb(16));
    /// assert_eq!(metadata.readonly, false);
    /// ```
    fn metadata(&self) -> PhysicalMemoryMetadata;

    // read helpers
    fn phys_read_raw_into(&mut self, addr: PhysicalAddress, out: &mut [u8]) -> Result<()> {
        self.phys_read_raw_list(&mut [PhysicalReadData(addr, out)])
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
        self.phys_write_raw_list(&[PhysicalWriteData(addr, data)])
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
impl<T: PhysicalMemory + ?Sized, P: std::ops::DerefMut<Target = T> + Send> PhysicalMemory for P {
    #[inline]
    fn phys_read_raw_list(&mut self, data: &mut [PhysicalReadData]) -> Result<()> {
        (**self).phys_read_raw_list(data)
    }

    #[inline]
    fn phys_write_raw_list(&mut self, data: &[PhysicalWriteData]) -> Result<()> {
        (**self).phys_write_raw_list(data)
    }

    #[inline]
    fn metadata(&self) -> PhysicalMemoryMetadata {
        (**self).metadata()
    }
}

/// Wrapper trait around physical memory which implements a boxed clone
pub trait CloneablePhysicalMemory: PhysicalMemory {
    fn clone_box(&self) -> Box<dyn CloneablePhysicalMemory>;
    fn downcast(&mut self) -> &mut dyn PhysicalMemory;
}

/// A sized Box containing a CloneablePhysicalMemory
pub type PhysicalMemoryBox = Box<dyn CloneablePhysicalMemory>;

/// Forward implementation of CloneablePhysicalMemory for every Cloneable backend.
impl<T> CloneablePhysicalMemory for T
where
    T: PhysicalMemory + Clone + 'static,
{
    fn clone_box(&self) -> PhysicalMemoryBox {
        Box::new(self.clone())
    }

    fn downcast(&mut self) -> &mut dyn PhysicalMemory {
        self
    }
}

/// Clone forward implementation for a PhysicalMemory Box
impl Clone for PhysicalMemoryBox {
    fn clone(&self) -> Self {
        (**self).clone_box()
    }
}

#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[repr(C)]
pub struct PhysicalMemoryMetadata {
    pub size: usize,
    pub readonly: bool,
}

// iterator helpers
#[repr(C)]
pub struct PhysicalReadData<'a>(pub PhysicalAddress, pub &'a mut [u8]);
pub trait PhysicalReadIterator<'a>: Iterator<Item = PhysicalReadData<'a>> + 'a {}
impl<'a, T: Iterator<Item = PhysicalReadData<'a>> + 'a> PhysicalReadIterator<'a> for T {}

impl<'a> From<PhysicalReadData<'a>> for (PhysicalAddress, &'a mut [u8]) {
    fn from(PhysicalReadData(a, b): PhysicalReadData<'a>) -> Self {
        (a, b)
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct PhysicalWriteData<'a>(pub PhysicalAddress, pub &'a [u8]);
pub trait PhysicalWriteIterator<'a>: Iterator<Item = PhysicalWriteData<'a>> + 'a {}
impl<'a, T: Iterator<Item = PhysicalWriteData<'a>> + 'a> PhysicalWriteIterator<'a> for T {}

impl<'a> From<PhysicalWriteData<'a>> for (PhysicalAddress, &'a [u8]) {
    fn from(PhysicalWriteData(a, b): PhysicalWriteData<'a>) -> Self {
        (a, b)
    }
}
