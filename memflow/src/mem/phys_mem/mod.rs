use crate::cglue::*;
use crate::dataview::Pod;
use crate::error::Result;
use crate::types::{umem, Address, PhysicalAddress};

use super::mem_data::*;
use super::PhysicalMemoryMapping;

use std::prelude::v1::*;

use crate::mem::memory_view::*;

pub mod cache;

pub use cache::*;

// TODO:
// - check endianess here and return an error
// - better would be to convert endianess with word alignment from addr

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
/// use std::convert::TryInto;
///
/// use memflow::mem::{
///     MemoryMap,
///     PhysicalMemoryMapping,
///     MemData,
///     phys_mem::{
///         PhysicalMemory,
///         PhysicalMemoryMetadata,
///     },
///     mem_data::{
///         PhysicalReadData,
///         PhysicalWriteData,
///         ReadFailCallback,
///         WriteFailCallback,
///     }
/// };
///
/// use memflow::cglue::CIterator;
///
/// use memflow::types::{PhysicalAddress, Address, umem};
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
///         _: &mut ReadFailCallback<'_, 'a>
///     ) -> Result<()> {
///         data
///             .for_each(|MemData(addr, mut out)| {
///                 let addr: usize = addr.to_umem().try_into().unwrap();
///                 let len = out.len();
///                 out.copy_from_slice(&self.mem[addr..(addr + len)])
///             });
///         Ok(())
///     }
///
///     fn phys_write_raw_iter<'a>(
///         &mut self,
///         data: CIterator<PhysicalWriteData<'a>>,
///         _: &mut WriteFailCallback<'_, 'a>
///     ) -> Result<()> {
///         data
///             .for_each(|MemData(addr, data)| {
///                 let addr: usize = addr.to_umem().try_into().unwrap();
///                 let len = data.len();
///                 self.mem[addr..(addr + len)].copy_from_slice(&data)
///             });
///         Ok(())
///     }
///
///     fn metadata(&self) -> PhysicalMemoryMetadata {
///         PhysicalMemoryMetadata {
///             max_address: (self.mem.len() - 1).into(),
///             real_size: self.mem.len() as umem,
///             readonly: false,
///             ideal_batch_size: u32::MAX
///         }
///     }
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
        out_fail: &mut ReadFailCallback<'_, 'a>,
    ) -> Result<()>;
    fn phys_write_raw_iter<'a>(
        &mut self,
        data: CIterator<PhysicalWriteData<'a>>,
        out_fail: &mut WriteFailCallback<'_, 'a>,
    ) -> Result<()>;

    /// Retrieve metadata about the physical memory
    ///
    /// This function will return metadata about the underlying physical memory object, currently
    /// including address space size and read-only status.
    ///
    /// # Examples
    ///
    /// ```
    /// use memflow::types::{size, mem};
    /// use memflow::mem::PhysicalMemory;
    /// # let mem = memflow::dummy::DummyMemory::new(size::mb(16));
    ///
    /// let metadata = mem.metadata();
    ///
    /// assert_eq!(metadata.max_address.to_umem(), mem::mb(16) - 1);
    /// assert_eq!(metadata.real_size, mem::mb(16));
    /// assert_eq!(metadata.readonly, false);
    /// ```
    fn metadata(&self) -> PhysicalMemoryMetadata;

    /// Sets the memory mapping for the physical memory
    ///
    /// In case a connector cannot acquire memory mappings on it's own this function
    /// allows the OS plugin to set the memory mapping at a later stage of initialization.
    ///
    /// The only reason this is needed for some connectors is to avoid catastrophic failures upon reading invalid address.
    ///
    /// By default this is a no-op.
    #[inline]
    fn set_mem_map(&mut self, _mem_map: &[PhysicalMemoryMapping]) {}

    #[skip_func]
    fn phys_read_into<T: Pod + ?Sized>(&mut self, addr: PhysicalAddress, out: &mut T) -> Result<()>
    where
        Self: Sized,
    {
        let mut iter = Some(MemData(addr, out.as_bytes_mut().into())).into_iter();
        self.phys_read_raw_iter(
            (&mut iter).into(),
            &mut (&mut |MemData(_, mut d): ReadData| {
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
        let mut iter = Some(MemData(addr, data.as_bytes().into())).into_iter();
        self.phys_write_raw_iter((&mut iter).into(), &mut (&mut |_| true).into())
    }

    #[vtbl_only('static, wrap_with_obj(MemoryView))]
    fn into_phys_view(self) -> PhysicalMemoryView<Self>
    where
        Self: Sized,
    {
        PhysicalMemoryView { mem: self }
    }

    #[vtbl_only('_, wrap_with_obj(MemoryView))]
    fn phys_view(&mut self) -> PhysicalMemoryView<Fwd<&mut Self>>
    where
        Self: Sized,
    {
        self.forward_mut().into_phys_view()
    }
}

#[repr(C)]
#[derive(Clone)]
#[cfg_attr(feature = "abi_stable", derive(::abi_stable::StableAbi))]
pub struct PhysicalMemoryView<T> {
    mem: T,
}

impl<T: PhysicalMemory> MemoryView for PhysicalMemoryView<T> {
    fn read_raw_iter<'a>(
        &mut self,
        data: CIterator<ReadData<'a>>,
        out_fail: &mut ReadFailCallback<'_, 'a>,
    ) -> Result<()> {
        let mut iter = data.map(|MemData(addr, data)| MemData(addr.into(), data));
        self.mem.phys_read_raw_iter((&mut iter).into(), out_fail)
    }

    fn write_raw_iter<'a>(
        &mut self,
        data: CIterator<WriteData<'a>>,
        out_fail: &mut WriteFailCallback<'_, 'a>,
    ) -> Result<()> {
        let mut iter = data.map(|MemData(addr, data)| MemData(addr.into(), data));
        self.mem.phys_write_raw_iter((&mut iter).into(), out_fail)
    }

    fn metadata(&self) -> MemoryViewMetadata {
        let PhysicalMemoryMetadata {
            max_address,
            real_size,
            readonly,
            ..
        } = self.mem.metadata();

        MemoryViewMetadata {
            max_address,
            real_size,
            readonly,
            #[cfg(target_pointer_width = "64")]
            arch_bits: 64,
            #[cfg(target_pointer_width = "32")]
            arch_bits: 32,
            #[cfg(target_endian = "little")]
            little_endian: true,
            #[cfg(target_endian = "big")]
            little_endian: false,
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[cfg_attr(feature = "abi_stable", derive(::abi_stable::StableAbi))]
pub struct PhysicalMemoryMetadata {
    pub max_address: Address,
    pub real_size: umem,
    pub readonly: bool,
    pub ideal_batch_size: u32,
}
