use crate::cglue::*;
use crate::dataview::{Pod, PodMethods};
use crate::error::Result;
use crate::types::{umem, Address, PhysicalAddress};

use super::mem_data::*;
use super::PhysicalMemoryMapping;

use std::prelude::v1::*;

use crate::mem::memory_view::*;

pub mod middleware;

pub use middleware::*;

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
///     phys_mem::{
///         PhysicalMemory,
///         PhysicalMemoryMetadata,
///     },
///     mem_data::{
///         MemOps,
///         PhysicalReadMemOps,
///         PhysicalWriteMemOps,
///         opt_call,
///     }
/// };
///
/// use memflow::cglue::{CIterator, CTup2, CTup3};
///
/// use memflow::types::{PhysicalAddress, Address, umem};
/// use memflow::error::Result;
///
/// pub struct MemoryBackend {
///     mem: Box<[u8]>,
/// }
///
/// impl PhysicalMemory for MemoryBackend {
///     fn phys_read_raw_iter(
///         &mut self,
///         MemOps {
///             inp,
///             mut out,
///             ..
///         }: PhysicalReadMemOps,
///     ) -> Result<()> {
///         inp
///             .for_each(|CTup3(addr, meta_addr, mut data)| {
///                 let addr: usize = addr.to_umem().try_into().unwrap();
///                 let len = data.len();
///                 data.copy_from_slice(&self.mem[addr..(addr + len)]);
///                 opt_call(out.as_deref_mut(), CTup2(meta_addr, data));
///             });
///         Ok(())
///     }
///
///     fn phys_write_raw_iter(
///         &mut self,
///         MemOps {
///             inp,
///             mut out,
///             ..
///         }: PhysicalWriteMemOps,
///     ) -> Result<()> {
///         inp
///             .for_each(|CTup3(addr, meta_addr, data)| {
///                 let addr: usize = addr.to_umem().try_into().unwrap();
///                 let len = data.len();
///                 self.mem[addr..(addr + len)].copy_from_slice(&data);
///                 opt_call(out.as_deref_mut(), CTup2(meta_addr, data));
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
    fn phys_read_raw_iter(&mut self, data: PhysicalReadMemOps) -> Result<()>;
    fn phys_write_raw_iter(&mut self, data: PhysicalWriteMemOps) -> Result<()>;

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
        MemOps::with(
            std::iter::once((addr, CSliceMut::from(out.as_bytes_mut()))),
            None,
            Some(
                &mut (&mut |CTup2(_, mut d): ReadData| {
                    d.iter_mut().for_each(|b| *b = 0);
                    true
                })
                    .into(),
            ),
            |data| self.phys_read_raw_iter(data),
        )
    }

    #[skip_func]
    fn phys_write<T: Pod + ?Sized>(&mut self, addr: PhysicalAddress, data: &T) -> Result<()>
    where
        Self: Sized,
    {
        MemOps::with(
            std::iter::once((addr, CSliceRef::from(data.as_bytes()))),
            None,
            None,
            |data| self.phys_write_raw_iter(data),
        )
    }

    // deprecated = Remove this function (superseeded by into_mem_view)
    #[vtbl_only('static, wrap_with_obj(MemoryView))]
    fn into_phys_view(self) -> PhysicalMemoryView<Self>
    where
        Self: Sized,
    {
        PhysicalMemoryView {
            mem: self,
            zero_fill_gaps: false,
        }
    }

    // deprecated = Remove this function (superseeded by mem_view)
    #[vtbl_only('_, wrap_with_obj(MemoryView))]
    fn phys_view(&mut self) -> PhysicalMemoryView<Fwd<&mut Self>>
    where
        Self: Sized,
    {
        self.forward_mut().into_mem_view()
    }

    // deprecated = Expose this via cglue
    #[skip_func]
    //#[vtbl_only('static, wrap_with_obj(MemoryView))]
    fn into_mem_view(self) -> PhysicalMemoryView<Self>
    where
        Self: Sized,
    {
        PhysicalMemoryView {
            mem: self,
            zero_fill_gaps: false,
        }
    }

    // deprecated = Expose this via cglue
    #[skip_func]
    //#[vtbl_only('_, wrap_with_obj(MemoryView))]
    fn mem_view(&mut self) -> PhysicalMemoryView<Fwd<&mut Self>>
    where
        Self: Sized,
    {
        self.forward_mut().into_mem_view()
    }
}

#[repr(C)]
#[derive(Clone)]
#[cfg_attr(feature = "abi_stable", derive(::abi_stable::StableAbi))]
pub struct PhysicalMemoryView<T> {
    mem: T,
    zero_fill_gaps: bool,
}

impl<T> PhysicalMemoryView<T> {
    pub fn zero_fill_gaps(mut self) -> Self {
        self.zero_fill_gaps = true;
        self
    }
}

impl<T: PhysicalMemory> MemoryView for PhysicalMemoryView<T> {
    fn read_raw_iter<'a>(
        &mut self,
        MemOps { inp, out, out_fail }: ReadRawMemOps<'a, '_, '_, '_>,
    ) -> Result<()> {
        let inp = &mut inp.map(|CTup3(addr, meta_addr, data)| CTup3(addr.into(), meta_addr, data));
        let inp = inp.into();

        #[allow(clippy::unnecessary_unwrap)]
        if self.zero_fill_gaps && out.is_some() && out_fail.is_some() {
            let out = std::cell::RefCell::new(out.unwrap());

            let ma = self.mem.metadata().max_address;

            let out1 = &mut |data| out.borrow_mut().call(data);
            let out = &mut |data| out.borrow_mut().call(data);
            let out = &mut out.into();
            let out = Some(out);

            let out_fail = out_fail.unwrap();

            let out_fail = &mut |mut data: ReadData<'a>| {
                if data.0 < ma {
                    data.1.iter_mut().for_each(|b| *b = 0);
                    out1(data)
                } else {
                    out_fail.call(data)
                }
            };

            let out_fail = &mut out_fail.into();
            let out_fail = Some(out_fail);

            let data = MemOps { inp, out, out_fail };
            self.mem.phys_read_raw_iter(data)
        } else {
            let data = MemOps { inp, out, out_fail };
            self.mem.phys_read_raw_iter(data)
        }
    }

    fn write_raw_iter(&mut self, MemOps { inp, out, out_fail }: WriteRawMemOps) -> Result<()> {
        let inp = &mut inp.map(|CTup3(addr, meta_addr, data)| CTup3(addr.into(), meta_addr, data));
        let inp = inp.into();

        let data = MemOps { inp, out, out_fail };

        self.mem.phys_write_raw_iter(data)
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
