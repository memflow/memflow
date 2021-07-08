use crate::cglue::*;
use crate::dataview::Pod;
use crate::error::Result;
use crate::types::{Address, PhysicalAddress};

use super::PhysicalMemoryMapping;

use std::prelude::v1::*;

use crate::mem::memory_view::*;

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
///     MemoryMap,
///     PhysicalMemoryMapping,
///     phys_mem::{
///         PhysicalMemory,
///         PhysicalReadData,
///         PhysicalWriteData,
///         PhysicalReadFailCallback,
///         PhysicalWriteFailCallback,
///         PhysicalMemoryMetadata,
///     }
/// };
///
/// use memflow::cglue::CIterator;
///
/// use memflow::types::{PhysicalAddress, Address};
/// use memflow::error::Result;
///
/// pub struct MemoryBackend {
///     mem: Box<[u8]>,
/// }
///
/// impl PhysicalMemory for MemoryBackend {
///     fn phys_read_raw_iter<'a>(
///         &mut self,
///         data: CIterator<PhysicalReadData<'a>>,
///         _: &mut PhysicalReadFailCallback<'_, 'a>
///     ) -> Result<()> {
///         data
///             .for_each(|PhysicalReadData(addr, mut out)| {
///                 let len = out.len();
///                 out.copy_from_slice(&self.mem[addr.as_usize()..(addr.as_usize() + len)])
///             });
///         Ok(())
///     }
///
///     fn phys_write_raw_iter<'a>(
///         &mut self,
///         data: CIterator<PhysicalWriteData<'a>>,
///         _: &mut PhysicalWriteFailCallback<'_, 'a>
///     ) -> Result<()> {
///         data
///             .for_each(|PhysicalWriteData(addr, data)| self
///                 .mem[addr.as_usize()..(addr.as_usize() + data.len())].copy_from_slice(&data)
///             );
///         Ok(())
///     }
///
///     fn metadata(&self) -> PhysicalMemoryMetadata {
///         PhysicalMemoryMetadata {
///             max_address: (self.mem.len() - 1).into(),
///             real_size: self.mem.len() as u64,
///             readonly: false,
///             ideal_batch_size: u32::MAX
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
    fn phys_read_raw_iter<'a>(
        &mut self,
        data: CIterator<PhysicalReadData<'a>>,
        out_fail: &mut PhysicalReadFailCallback<'_, 'a>,
    ) -> Result<()>;
    fn phys_write_raw_iter<'a>(
        &mut self,
        data: CIterator<PhysicalWriteData<'a>>,
        out_fail: &mut PhysicalWriteFailCallback<'_, 'a>,
    ) -> Result<()>;

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

    #[skip_func]
    fn phys_read_into<T: Pod + ?Sized>(&mut self, addr: PhysicalAddress, out: &mut T) -> Result<()>
    where
        Self: Sized,
    {
        let mut iter = Some(PhysicalReadData(addr, out.as_bytes_mut().into())).into_iter();
        self.phys_read_raw_iter(
            (&mut iter).into(),
            &mut (&mut |PhysicalReadData(_, mut d)| {
                d.iter_mut().for_each(|b| *b = 0);
                true
            })
                .into(),
        )
    }

    #[skip_func]
    fn phys_write<T: Pod + ?Sized>(&mut self, addr: PhysicalAddress, data: &T) -> Result<()>
    where
        Self: Sized,
    {
        println!("{:x}", data.as_bytes().len());
        let mut iter = Some(PhysicalWriteData(addr, data.as_bytes().into())).into_iter();
        self.phys_write_raw_iter((&mut iter).into(), &mut (&mut |_| false).into())
    }

    // TODO: create FFI helpers for this
    #[skip_func]
    fn into_phys_view(self) -> PhysicalMemoryView<Self>
    where
        Self: Sized,
    {
        PhysicalMemoryView { mem: self }
    }

    #[skip_func]
    fn phys_view(&mut self) -> PhysicalMemoryView<Fwd<&mut Self>>
    where
        Self: Sized,
    {
        self.forward_mut().into_phys_view()
    }
}

#[repr(C)]
pub struct PhysicalMemoryView<T> {
    mem: T,
}

impl<T: PhysicalMemory> MemoryView for PhysicalMemoryView<T> {
    fn read_raw_iter<'a>(
        &mut self,
        data: CIterator<ReadData<'a>>,
        out_fail: &mut ReadFailCallback<'_, 'a>,
    ) -> Result<()> {
        let mut iter = data.map(|ReadData(addr, data)| PhysicalReadData(addr.into(), data));

        let mut callback =
            |PhysicalReadData(addr, data)| out_fail.call(ReadData(addr.address(), (&data).into()));
        let callback = &mut callback;

        self.mem
            .phys_read_raw_iter((&mut iter).into(), &mut callback.into())
    }

    fn write_raw_iter<'a>(
        &mut self,
        data: CIterator<WriteData<'a>>,
        out_fail: &mut WriteFailCallback<'_, 'a>,
    ) -> Result<()> {
        let mut iter = data.map(|WriteData(addr, data)| PhysicalWriteData(addr.into(), data));

        let mut callback =
            |PhysicalWriteData(addr, data)| out_fail.call(WriteData(addr.address(), data));
        let callback = &mut callback;

        self.mem
            .phys_write_raw_iter((&mut iter).into(), &mut callback.into())
    }

    fn metadata(&self) -> MemoryViewMetadata {
        let PhysicalMemoryMetadata {
            max_address,
            real_size,
            readonly,
            ideal_batch_size,
        } = self.mem.metadata();

        MemoryViewMetadata {
            max_address,
            real_size,
            readonly,
            ideal_batch_size,
        }
    }
}

#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "'serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[repr(C)]
pub struct PhysicalMemoryMetadata {
    pub max_address: Address,
    pub real_size: u64,
    pub readonly: bool,
    pub ideal_batch_size: u32,
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

pub type PhysicalReadFailCallback<'a, 'b> = OpaqueCallback<'a, PhysicalReadData<'b>>;

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

pub type PhysicalWriteFailCallback<'a, 'b> = OpaqueCallback<'a, PhysicalWriteData<'b>>;
