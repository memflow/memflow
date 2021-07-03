use crate::cglue::*;
use crate::dataview::Pod;
use crate::error::{Error, ErrorKind, ErrorOrigin, Result};
use crate::types::{Address, PhysicalAddress, Pointer32, Pointer64};

use super::{PhysicalMemoryBatcher, PhysicalMemoryMapping};

use std::mem::MaybeUninit;
use std::prelude::v1::*;

#[cfg(feature = "std")]
use super::PhysicalMemoryCursor;

// those only required when compiling cglue code
#[cfg(feature = "plugins")]
use crate::connector::cpu_state::*;

// TODO:
// - check endianess here and return an error
// - better would be to convert endianess with word alignment from addr

#[cfg(feature = "plugins")]
cglue_trait_group!(ConnectorInstance<'a>, { PhysicalMemory, Clone }, { ConnectorCpuStateInner<'a> });
#[cfg(feature = "plugins")]
pub type MuConnectorInstanceArcBox<'a> = std::mem::MaybeUninit<ConnectorInstanceArcBox<'a>>;

/// The [`PhysicalMemory`] trait is implemented by memory backends
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
/// Implementing [`PhysicalMemory`] for a memory backend:
/// ```
/// use std::vec::Vec;
///
/// use memflow::mem::{
///     PhysicalMemory,
///     PhysicalReadData,
///     PhysicalWriteData,
///     PhysicalMemoryMetadata,
///     PhysicalMemoryMapping,
///     MemoryMap
/// };
///
/// use memflow::types::{PhysicalAddress, Address};
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
///             .for_each(|PhysicalReadData(addr, out)| {
///                 let len = out.len();
///                 out.copy_from_slice(&self.mem[addr.as_usize()..(addr.as_usize() + len)])
///             });
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
///             max_address: (self.mem.len() - 1).into(),
///             real_size: self.mem.len() as u64,
///             readonly: false
///         }
///     }
///
///     // this is a no-op in this example
///     fn set_mem_map(&mut self, _mem_map: &[PhysicalMemoryMapping]) {}
/// }
/// ```
///
/// Reading from [`PhysicalMemory`]:
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
/// # use memflow::dummy::DummyMemory;
/// # use memflow::types::size;
/// # read(&mut DummyMemory::new(size::mb(4)));
/// ```
#[cfg_attr(feature = "plugins", cglue_trait)]
#[int_result]
#[cglue_forward]
pub trait PhysicalMemory: Send {
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
    /// # let mem = memflow::dummy::DummyMemory::new(size::mb(16));
    ///
    /// let metadata = mem.metadata();
    ///
    /// assert_eq!(metadata.max_address.as_usize(), size::mb(16) - 1);
    /// assert_eq!(metadata.real_size, size::mb(16) as u64);
    /// assert_eq!(metadata.readonly, false);
    /// ```
    fn metadata(&self) -> PhysicalMemoryMetadata;

    /// Sets the memory mapping for the physical memory
    ///
    /// In case a connector cannot acquire memory mappings on it's own this function
    /// allows the OS plugin to set the memory mapping at a later stage of initialization.
    fn set_mem_map(&mut self, mem_map: &[PhysicalMemoryMapping]);

    // read helpers
    fn phys_read_raw_into(&mut self, addr: PhysicalAddress, out: &mut [u8]) -> Result<()> {
        self.phys_read_raw_list(&mut [PhysicalReadData(addr, out.into())])
    }

    #[skip_func]
    fn phys_read_into<T: Pod + ?Sized>(&mut self, addr: PhysicalAddress, out: &mut T) -> Result<()>
    where
        Self: Sized,
    {
        self.phys_read_raw_into(addr, out.as_bytes_mut())
    }

    #[skip_func]
    fn phys_read_raw(&mut self, addr: PhysicalAddress, len: usize) -> Result<Vec<u8>> {
        let mut buf = vec![0u8; len];
        self.phys_read_raw_into(addr, &mut *buf)?;
        Ok(buf)
    }

    /// # Safety
    ///
    /// this function will overwrite the contents of 'obj' so we can just allocate an unitialized memory section.
    /// this function should only be used with [repr(C)] structs.
    #[skip_func]
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
        self.phys_write_raw_list(&[PhysicalWriteData(addr, data.into())])
    }

    #[skip_func]
    fn phys_write<T: Pod + ?Sized>(&mut self, addr: PhysicalAddress, data: &T) -> Result<()>
    where
        Self: Sized,
    {
        self.phys_write_raw(addr, data.as_bytes())
    }

    // read pointer wrappers
    #[skip_func]
    fn phys_read_ptr32_into<U: Pod + ?Sized>(
        &mut self,
        ptr: Pointer32<U>,
        out: &mut U,
    ) -> Result<()>
    where
        Self: Sized,
    {
        self.phys_read_into(ptr.address.into(), out)
    }

    #[skip_func]
    fn phys_read_ptr32<U: Pod + Sized>(&mut self, ptr: Pointer32<U>) -> Result<U>
    where
        Self: Sized,
    {
        self.phys_read(ptr.address.into())
    }

    #[skip_func]
    fn phys_read_ptr64_into<U: Pod + ?Sized>(
        &mut self,
        ptr: Pointer64<U>,
        out: &mut U,
    ) -> Result<()>
    where
        Self: Sized,
    {
        self.phys_read_into(ptr.address.into(), out)
    }

    #[skip_func]
    fn phys_read_ptr64<U: Pod + Sized>(&mut self, ptr: Pointer64<U>) -> Result<U>
    where
        Self: Sized,
    {
        self.phys_read(ptr.address.into())
    }

    // write pointer wrappers
    #[skip_func]
    fn phys_write_ptr32<U: Pod + Sized>(&mut self, ptr: Pointer32<U>, data: &U) -> Result<()>
    where
        Self: Sized,
    {
        self.phys_write(ptr.address.into(), data)
    }

    #[skip_func]
    fn phys_write_ptr64<U: Pod + Sized>(&mut self, ptr: Pointer64<U>, data: &U) -> Result<()>
    where
        Self: Sized,
    {
        self.phys_write(ptr.address.into(), data)
    }

    /// Reads a fixed length string from the target.
    ///
    /// # Remarks:
    ///
    /// The string does not have to be null-terminated.
    /// If a null terminator is found the string is truncated to the terminator.
    /// If no null terminator is found the resulting string is exactly `len` characters long.
    #[skip_func]
    fn phys_read_char_array(&mut self, addr: PhysicalAddress, len: usize) -> Result<String> {
        let mut buf = vec![0; len];
        self.phys_read_raw_into(addr, &mut buf)?;
        if let Some((n, _)) = buf.iter().enumerate().find(|(_, c)| **c == 0_u8) {
            buf.truncate(n);
        }
        Ok(String::from_utf8_lossy(&buf).to_string())
    }

    /// Reads a variable length string with a length of up to specified amount from the target.
    ///
    /// # Arguments
    ///
    /// * `addr` - target address to read from
    /// * `n` - maximum number of bytes to read
    ///
    /// # Remarks:
    ///
    /// The string must be null-terminated.
    /// If no null terminator is found the this function will return an error.
    ///
    /// For reading fixed-size char arrays the [`virt_read_char_array`] should be used.
    #[skip_func]
    fn phys_read_char_string_n(&mut self, addr: PhysicalAddress, n: usize) -> Result<String> {
        let mut buf = vec![0; 32];

        let mut last_n = 0;

        loop {
            let (_, right) = buf.split_at_mut(last_n);

            // TODO: add a special add function which will check page boundaries and keep/destroy metadata
            self.phys_read_raw_into((addr.address() + last_n).into(), right)?;
            if let Some((n, _)) = right.iter().enumerate().find(|(_, c)| **c == 0_u8) {
                buf.truncate(last_n + n);
                return Ok(String::from_utf8_lossy(&buf).to_string());
            }
            if buf.len() >= n {
                break;
            }
            last_n = buf.len();

            buf.extend((0..buf.len()).map(|_| 0));
        }

        Err(Error(ErrorOrigin::PhysicalMemory, ErrorKind::OutOfBounds))
    }

    /// Reads a variable length string with up to 4kb length from the target.
    ///
    /// # Arguments
    ///
    /// * `addr` - target address to read from
    #[skip_func]
    fn phys_read_char_string(&mut self, addr: PhysicalAddress) -> Result<String> {
        self.phys_read_char_string_n(addr, 4096)
    }

    #[skip_func]
    fn phys_batcher(&mut self) -> PhysicalMemoryBatcher<Self>
    where
        Self: Sized,
    {
        PhysicalMemoryBatcher::new(self)
    }

    #[cfg(feature = "std")]
    #[skip_func]
    fn phys_cursor(&mut self) -> PhysicalMemoryCursor<Fwd<&mut Self>>
    where
        Self: Sized,
    {
        PhysicalMemoryCursor::new(self.forward())
    }

    #[cfg(feature = "std")]
    #[skip_func]
    fn into_phys_cursor(self) -> PhysicalMemoryCursor<Self>
    where
        Self: Sized,
    {
        PhysicalMemoryCursor::new(self)
    }

    #[cfg(feature = "std")]
    #[skip_func]
    fn phys_cursor_at(&mut self, address: PhysicalAddress) -> PhysicalMemoryCursor<Fwd<&mut Self>>
    where
        Self: Sized,
    {
        PhysicalMemoryCursor::at(self.forward(), address)
    }

    #[cfg(feature = "std")]
    #[skip_func]
    fn into_phys_cursor_at(self, address: PhysicalAddress) -> PhysicalMemoryCursor<Self>
    where
        Self: Sized,
    {
        PhysicalMemoryCursor::at(self, address)
    }
}

#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[repr(C)]
pub struct PhysicalMemoryMetadata {
    pub max_address: Address,
    pub real_size: u64,
    pub readonly: bool,
}

// iterator helpers
#[repr(C)]
pub struct PhysicalReadData<'a>(pub PhysicalAddress, pub CSliceMut<'a, u8>);
pub trait PhysicalReadIterator<'a>: Iterator<Item = PhysicalReadData<'a>> + 'a {}
impl<'a, T: Iterator<Item = PhysicalReadData<'a>> + 'a> PhysicalReadIterator<'a> for T {}

impl<'a> From<PhysicalReadData<'a>> for (PhysicalAddress, &'a mut [u8]) {
    fn from(PhysicalReadData(a, b): PhysicalReadData<'a>) -> Self {
        (a, b.into())
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct PhysicalWriteData<'a>(pub PhysicalAddress, pub CSliceRef<'a, u8>);
pub trait PhysicalWriteIterator<'a>: Iterator<Item = PhysicalWriteData<'a>> + 'a {}
impl<'a, T: Iterator<Item = PhysicalWriteData<'a>> + 'a> PhysicalWriteIterator<'a> for T {}

impl<'a> From<PhysicalWriteData<'a>> for (PhysicalAddress, &'a [u8]) {
    fn from(PhysicalWriteData(a, b): PhysicalWriteData<'a>) -> Self {
        (a, b.into())
    }
}
