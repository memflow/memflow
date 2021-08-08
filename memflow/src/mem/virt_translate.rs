use std::prelude::v1::*;

use std::cmp::*;

use cglue::prelude::v1::*;
use itertools::Itertools;

pub mod direct_translate;
use crate::iter::SplitAtIndex;
pub use direct_translate::DirectTranslate;

#[cfg(test)]
mod tests;

use crate::error::{Result, *};

use crate::mem::{MemData, PhysicalMemory};
use crate::types::{umem, Address, Page, PhysicalAddress};

use crate::architecture::{VirtualTranslate3, VtopFailureCallback, VtopOutputCallback};

#[cglue_trait]
#[int_result]
pub trait VirtualTranslate: Send {
    fn virt_to_phys_list(
        &mut self,
        addrs: &[MemoryRange],
        out: VirtualTranslationCallback,
        out_fail: VirtualTranslationFailCallback,
    );

    fn virt_to_phys_range(
        &mut self,
        start: Address,
        end: Address,
        out: VirtualTranslationCallback,
    ) {
        self.virt_to_phys_list(
            &[MemoryRange {
                address: start,
                size: end - start,
            }],
            out,
            (&mut |_| true).into(),
        )
    }

    fn virt_translation_map_range(
        &mut self,
        start: Address,
        end: Address,
        out: VirtualTranslationCallback,
    ) {
        let mut set = std::collections::BTreeSet::new();

        self.virt_to_phys_range(
            start,
            end,
            (&mut |v| {
                set.insert(v);
                true
            })
                .into(),
        );

        set.into_iter()
            .coalesce(|a, b| {
                // TODO: Probably make the page size reflect the merge
                if b.in_virtual == (a.in_virtual + a.size)
                    && b.out_physical.address() == (a.out_physical.address() + a.size)
                    && a.out_physical.page_type() == b.out_physical.page_type()
                {
                    Ok(VirtualTranslation {
                        in_virtual: a.in_virtual,
                        size: a.size + b.size,
                        out_physical: a.out_physical,
                    })
                } else {
                    Err((a, b))
                }
            })
            .feed_into(out);
    }

    fn virt_page_map_range(
        &mut self,
        gap_size: umem,
        start: Address,
        end: Address,
        out: MemoryRangeCallback,
    ) {
        let mut set: rangemap::RangeSet<Address> = Default::default();

        self.virt_to_phys_range(
            start,
            end,
            (&mut |VirtualTranslation {
                       in_virtual,
                       size,
                       out_physical: _,
                   }| {
                set.insert(in_virtual..(in_virtual + size));
                true
            })
                .into(),
        );

        set.gaps(&(start..end))
            .filter(|r| r.end - r.start <= gap_size)
            .collect::<Vec<_>>()
            .into_iter()
            .for_each(|r| set.insert(r));

        set.iter()
            .map(|r| {
                let address = r.start;
                let size = r.end - address;
                MemoryRange { address, size }
            })
            .feed_into(out);
    }

    fn virt_to_phys(&mut self, address: Address) -> Result<PhysicalAddress> {
        let mut out = Err(Error(ErrorOrigin::VirtualTranslate, ErrorKind::OutOfBounds));

        self.virt_to_phys_list(
            &[MemoryRange { address, size: 1 }],
            (&mut |VirtualTranslation {
                       in_virtual: _,
                       size: _,
                       out_physical,
                   }| {
                out = Ok(out_physical);
                false
            })
                .into(),
            (&mut |_| true).into(),
        );

        out
    }

    fn virt_page_info(&mut self, addr: Address) -> Result<Page> {
        let paddr = self.virt_to_phys(addr)?;
        Ok(paddr.containing_page())
    }

    #[skip_func]
    fn virt_page_map_range_vec(
        &mut self,
        gap_size: umem,
        start: Address,
        end: Address,
    ) -> Vec<MemoryRange> {
        let mut out = vec![];
        self.virt_page_map_range(gap_size, start, end, (&mut out).into());
        out
    }

    // page map helpers
    fn virt_translation_map(&mut self, out: VirtualTranslationCallback) {
        self.virt_translation_map_range(Address::null(), Address::invalid(), out)
    }

    #[skip_func]
    fn virt_translation_map_vec(&mut self) -> Vec<VirtualTranslation> {
        let mut out = vec![];
        self.virt_translation_map((&mut out).into());
        out
    }

    /// Attempt to translate a physical address into a virtual one.
    ///
    /// This function is the reverse of [`virt_to_phys`](VirtualTranslate::virt_to_phys). Note, that there could be multiple virtual
    /// addresses for one physical address. If all candidates are needed, use
    /// [`phys_to_virt_vec`](VirtualTranslate::phys_to_virt_vec) function.
    fn phys_to_virt(&mut self, phys: Address) -> Option<Address> {
        let mut virt = None;

        let callback = &mut |VirtualTranslation {
                                 in_virtual,
                                 size: _,
                                 out_physical,
                             }| {
            if out_physical.address() == phys {
                virt = Some(in_virtual);
                false
            } else {
                true
            }
        };

        self.virt_translation_map(callback.into());

        virt
    }

    /// Retrieve all virtual address that map into a given physical address.
    #[skip_func]
    fn phys_to_virt_vec(&mut self, phys: Address) -> Vec<Address> {
        let mut virt = vec![];

        let callback = &mut |VirtualTranslation {
                                 in_virtual,
                                 size: _,
                                 out_physical,
                             }| {
            if out_physical.address() == phys {
                virt.push(in_virtual);
                true
            } else {
                true
            }
        };

        self.virt_translation_map(callback.into());

        virt
    }

    fn virt_page_map(&mut self, gap_size: umem, out: MemoryRangeCallback) {
        self.virt_page_map_range(gap_size, Address::null(), Address::invalid(), out)
    }

    #[skip_func]
    fn virt_page_map_vec(&mut self, gap_size: umem) -> Vec<MemoryRange> {
        let mut out = vec![];
        self.virt_page_map(gap_size, (&mut out).into());
        out
    }
}

pub type VirtualTranslationCallback<'a> = OpaqueCallback<'a, VirtualTranslation>;
pub type MemoryRangeCallback<'a> = OpaqueCallback<'a, MemoryRange>;
pub type VirtualTranslationFailCallback<'a> = OpaqueCallback<'a, VirtualTranslationFail>;

/// Virtual page range information with physical mappings used for callbacks
#[repr(C)]
#[derive(Clone, Debug, Eq, Copy)]
pub struct VirtualTranslation {
    pub in_virtual: Address,
    pub size: umem,
    pub out_physical: PhysicalAddress,
}

impl Ord for VirtualTranslation {
    fn cmp(&self, other: &Self) -> Ordering {
        self.in_virtual.cmp(&other.in_virtual)
    }
}

impl PartialOrd for VirtualTranslation {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for VirtualTranslation {
    fn eq(&self, other: &Self) -> bool {
        self.in_virtual == other.in_virtual
    }
}

/// Virtual page range information used for callbacks
#[repr(C)]
#[derive(Clone, Debug, PartialEq, Copy)]
pub struct MemoryRange {
    pub address: Address,
    pub size: umem,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct VirtualTranslationFail {
    pub from: Address,
    pub size: umem,
}

pub trait VirtualTranslate2
where
    Self: Send,
{
    /// Translate a list of virtual addresses
    ///
    /// This function will do a virtual to physical memory translation for the
    /// `VirtualTranslate3` over multiple elements.
    ///
    /// In most cases, you will want to use the `VirtualDma`, but this trait is provided if needed
    /// to implement some more advanced filtering.
    ///
    /// # Examples
    ///
    /// ```
    /// # use memflow::error::Result;
    /// # use memflow::types::{PhysicalAddress, Address};
    /// # use memflow::dummy::{DummyMemory, DummyOs};
    /// use memflow::mem::{VirtualTranslate2, DirectTranslate, MemData};
    /// use memflow::types::size;
    /// use memflow::architecture::x86::x64;
    /// use memflow::cglue::FromExtend;
    ///
    /// # const VIRT_MEM_SIZE: usize = size::mb(8);
    /// # const CHUNK_SIZE: usize = 2;
    /// #
    /// # let mem = DummyMemory::new(size::mb(16));
    /// # let mut os = DummyOs::new(mem);
    /// # let (dtb, virtual_base) = os.alloc_dtb(VIRT_MEM_SIZE, &[]);
    /// # let mut mem = os.into_inner();
    /// # let translator = x64::new_translator(dtb);
    /// let arch = x64::ARCH;
    ///
    /// let mut buffer = vec![0; VIRT_MEM_SIZE * CHUNK_SIZE / arch.page_size()];
    /// let buffer_length = buffer.len();
    ///
    /// // In this example, 8 megabytes starting from `virtual_base` are mapped in.
    /// // We translate 2 bytes chunks over the page boundaries. These bytes will be
    /// // split off into 2 separate translated chunks.
    /// let addresses = buffer
    ///     .chunks_mut(CHUNK_SIZE)
    ///     .enumerate()
    ///     .map(|(i, buf)| MemData(virtual_base + ((i + 1) * size::kb(4) - 1), buf));
    ///
    /// let mut translated_data = vec![];
    /// let mut failed_translations = &mut |_| true;
    ///
    /// let mut direct_translate = DirectTranslate::new();
    ///
    /// direct_translate.virt_to_phys_iter(
    ///     &mut mem,
    ///     &translator,
    ///     addresses,
    ///     &mut translated_data.from_extend(),
    ///     &mut failed_translations.into(),
    /// );
    ///
    ///
    /// // We tried to translate one byte out of the mapped memory, it had to fail
    /// assert_eq!(translated_data.len(), buffer_length - 1);
    ///
    /// # Ok::<(), memflow::error::Error>(())
    /// ```
    fn virt_to_phys_iter<T, B, D, VI>(
        &mut self,
        phys_mem: &mut T,
        translator: &D,
        addrs: VI,
        out: &mut VtopOutputCallback<B>,
        out_fail: &mut VtopFailureCallback<B>,
    ) where
        T: PhysicalMemory + ?Sized,
        B: SplitAtIndex,
        D: VirtualTranslate3,
        VI: Iterator<Item = MemData<Address, B>>;

    /// Translate a single virtual address
    ///
    /// This function will do a virtual to physical memory translation for the
    /// `VirtualTranslate3` for single address returning either PhysicalAddress, or an error.
    ///
    /// # Examples
    /// ```
    /// # use memflow::error::Result;
    /// # use memflow::types::{PhysicalAddress, Address};
    /// # use memflow::dummy::{DummyMemory, DummyOs};
    /// # use memflow::types::size;
    /// # use memflow::architecture::VirtualTranslate3;
    /// use memflow::mem::{VirtualTranslate2, DirectTranslate};
    /// use memflow::architecture::x86::x64;
    ///
    /// # const VIRT_MEM_SIZE: usize = size::mb(8);
    /// # const CHUNK_SIZE: usize = 2;
    /// #
    /// # let mem = DummyMemory::new(size::mb(16));
    /// # let mut os = DummyOs::new(mem);
    /// # let (dtb, virtual_base) = os.alloc_dtb(VIRT_MEM_SIZE, &[]);
    /// # let mut mem = os.into_inner();
    /// # let translator = x64::new_translator(dtb);
    /// let arch = x64::ARCH;
    ///
    /// let mut direct_translate = DirectTranslate::new();
    ///
    /// // Translate a mapped address
    /// let res = direct_translate.virt_to_phys(
    ///     &mut mem,
    ///     &translator,
    ///     virtual_base,
    /// );
    ///
    /// assert!(res.is_ok());
    ///
    /// // Translate unmapped address
    /// let res = direct_translate.virt_to_phys(
    ///     &mut mem,
    ///     &translator,
    ///     virtual_base - 1,
    /// );
    ///
    /// assert!(res.is_err());
    ///
    /// ```
    fn virt_to_phys<T: PhysicalMemory + ?Sized, D: VirtualTranslate3>(
        &mut self,
        phys_mem: &mut T,
        translator: &D,
        vaddr: Address,
    ) -> Result<PhysicalAddress> {
        let mut output = None;
        let success = &mut |elem: MemData<PhysicalAddress, _>| {
            if output.is_none() {
                output = Some(elem.0);
            }
            false
        };
        let mut output_err = None;
        let fail = &mut |elem: (Error, _)| {
            output_err = Some(elem.0);
            true
        };

        self.virt_to_phys_iter(
            phys_mem,
            translator,
            Some(MemData(vaddr, 1 as umem)).into_iter(),
            &mut success.into(),
            &mut fail.into(),
        );
        output.map(Ok).unwrap_or_else(|| Err(output_err.unwrap()))
    }
}

// forward impls
impl<'a, T, P> VirtualTranslate2 for P
where
    T: VirtualTranslate2 + ?Sized,
    P: std::ops::DerefMut<Target = T> + Send,
{
    #[inline]
    fn virt_to_phys_iter<U, B, D, VI>(
        &mut self,
        phys_mem: &mut U,
        translator: &D,
        addrs: VI,
        out: &mut VtopOutputCallback<B>,
        out_fail: &mut VtopFailureCallback<B>,
    ) where
        U: PhysicalMemory + ?Sized,
        B: SplitAtIndex,
        D: VirtualTranslate3,
        VI: Iterator<Item = MemData<Address, B>>,
    {
        (**self).virt_to_phys_iter(phys_mem, translator, addrs, out, out_fail)
    }
}
