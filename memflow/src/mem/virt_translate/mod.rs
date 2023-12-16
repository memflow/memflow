//! Virtual address translation
//!
//! This module describes virtual to physical address translation interfaces.
//!
//! * [VirtualTranslate](VirtualTranslate) - user facing trait providing a way to translate
//! addresses.
//!
//! * [VirtualTranslate2](VirtualTranslate2) - internally used trait that translates pairs of
//! buffers and virtual addresses into pairs of buffers and their corresponding physical addresses.
//! Is used to provide [virtual memory view](crate::mem::virt_mem::virtual_dma). This trait is also
//! a [point of caching](crate::mem::virt_translate::cache) for the translations.
//!
//! * [VirtualTranslate3](VirtualTranslate3) - a sub-scope that translates addresses of a single
//! address space. Objects that implement VirtualTranslate3 are designed to be cheap to construct,
//! because they use pooled resources from VirtualTranslate2 objects. This is equivalent to storing
//! a single VirtualTranslate2 state for the OS, while constructing VirtualTranslate3 instances for
//! each process. This is precisely what is being done in our Win32 OS (see
//! [here](https://github.com/memflow/memflow-win32/blob/791bb7afb8a984034dde314c136b7675b44e3abf/src/win32/process.rs#L348),
//! and
//! [here](https://github.com/memflow/memflow-win32/blob/791bb7afb8a984034dde314c136b7675b44e3abf/src/win32/process.rs#L314)).
//!
//! Below figure shows entire pipeline of a virtual address translating object with caching.
//!
//! ```text
//! +--------------------------+
//! |      (Win32Process)      |
//! |     VirtualTranslate     | (Contains VT2+VT3+Phys)
//! |        MemoryView        |
//! +--------------------------+
//!             |
//!             |
//! +-----------+--------------+
//! | (CachedVirtualTranslate) | (Accepts VT3+Phys)
//! |    VirtualTranslate2     | (Point of caching)
//! +--------------------------+
//!             |
//!             |
//!    +--------+----------+
//!    | (DirectTranslate) | (Accepts VT3+Phys)
//!    | VirtualTranslate2 | (Contains 64MB buffer)
//!    +-------------------+
//!             |
//!             |
//!  +----------+-------------+
//!  | (X86 VirtualTranslate) | (Accepts 64MB buffer+Phys)
//!  |   VirtualTranslate3    | (Contains CR3+ArchMmuSpec)
//!  +------------------------+
//!             |
//!             |
//!      +------+------+
//!      | ArchMmuSpec | (Accepts translation root (CR3), buffer, Phys)
//!      +-------------+ (Contains architecture specification)
//!             |
//!             |
//!     +-------+--------+
//!     | PhysicalMemory | (Accepts special page flags)
//!     +----------------+
//!             |
//!             |
//!            ... (Further nesting)
//! ```

use std::prelude::v1::*;

use super::{MemoryRange, MemoryRangeCallback, VtopRange};

use std::cmp::*;

use cglue::prelude::v1::*;
use itertools::Itertools;

pub mod direct_translate;
use crate::iter::SplitAtIndex;
pub use direct_translate::DirectTranslate;

use crate::architecture::ArchitectureObj;
use crate::types::gap_remover::GapRemover;

#[macro_use]
pub mod mmu;

pub mod cache;

pub use cache::*;

#[cfg(test)]
mod tests;

use crate::error::{Result, *};

use crate::mem::PhysicalMemory;
use crate::types::{imem, umem, Address, Page, PhysicalAddress};

/// Translates virtual addresses into physical ones.
///
/// This is a simple user-facing trait to perform virtual address translations. Implementor needs
/// to implement only 1 function - [virt_to_phys_list](VirtualTranslate::virt_to_phys_list). Other
/// functions are provided as helpers built on top of the base function.
///
/// For overview how this trait relates to other virtual translation traits,
/// check out documentation of [this module](self).
#[cfg_attr(feature = "plugins", cglue_trait)]
#[int_result]
pub trait VirtualTranslate: Send {
    /// Translate a list of address ranges into physical address space.
    ///
    /// This function will take addresses in `addrs` and produce translation of them into `out`.
    /// Any unsuccessful ranges will be sent to `out_fail`.
    ///
    /// # Remarks
    ///
    /// Note that the number of outputs may not match the number of inputs - virtual address space
    /// does not usually map linearly to the physical one, thus the input may need to be split into
    /// smaller parts, which may not be combined back together.
    ///
    /// # Example:
    ///
    /// ```
    /// use memflow::prelude::v1::*;
    /// # use memflow::dummy::DummyOs;
    ///
    /// // Virtual translation test
    /// fn vtop(mem: &mut impl VirtualTranslate, addr: Address) {
    ///     let mut cnt = 0;
    ///     mem.virt_to_phys_list(
    ///         &[CTup2(addr, 0x2000)],
    ///         // Successfully translated
    ///         (&mut |_| { cnt += 1; true }).into(),
    ///         // Failed to translate
    ///         (&mut |v| panic!("Failed to translate: {:?}", v)).into()
    ///     );
    ///     // We attempt to translate 2 pages, thus there are 2 outputs.
    ///     assert_eq!(2, cnt);
    /// }
    /// # let mut proc = DummyOs::quick_process(size::mb(2), &[]);
    /// # let addr = proc.info().address;
    /// # vtop(&mut proc.mem, addr);
    /// ```
    fn virt_to_phys_list(
        &mut self,
        addrs: &[VtopRange],
        out: VirtualTranslationCallback,
        out_fail: VirtualTranslationFailCallback,
    );

    /// Translate a single virtual address range into physical address space.
    ///
    /// This function is a helper for [`virt_to_phys_list`](Self::virt_to_phys_list) that translates
    /// just a single range, and has no failure output. It is otherwise identical.
    ///
    /// # Example:
    ///
    /// ```
    /// use memflow::prelude::v1::*;
    /// # use memflow::dummy::DummyOs;
    ///
    /// // Virtual translation test
    /// fn vtop(mem: &mut impl VirtualTranslate, addr: Address) {
    ///     let mut cnt = 0;
    ///     mem.virt_to_phys_range(
    ///         addr, addr + 0x2000,
    ///         // Successfully translated
    ///         (&mut |_| { cnt += 1; true }).into(),
    ///     );
    ///     // We attempt to translate 2 pages, thus there are 2 outputs.
    ///     assert_eq!(2, cnt);
    /// }
    /// # let mut proc = DummyOs::quick_process(size::mb(2), &[]);
    /// # let addr = proc.info().address;
    /// # vtop(&mut proc.mem, addr);
    /// ```
    fn virt_to_phys_range(
        &mut self,
        start: Address,
        end: Address,
        out: VirtualTranslationCallback,
    ) {
        assert!(end >= start);
        self.virt_to_phys_list(
            &[CTup2(start, (end - start) as umem)],
            out,
            (&mut |_| true).into(),
        )
    }

    /// Translate a single virtual address range into physical address space and coalesce nearby
    /// regions.
    ///
    /// This function is nearly identical to [`virt_to_phys_range`](Self::virt_to_phys_range), however,
    /// it performs additional post-processing of the output to combine consecutive ranges, and
    /// output them in sorted order (by input virtual address).
    ///
    /// # Example:
    ///
    /// ```
    /// use memflow::prelude::v1::*;
    /// use memflow::dummy::{DummyOs, DummyMemory};
    ///
    /// // Create a dummy OS
    /// let mem = DummyMemory::new(size::mb(1));
    /// let mut os = DummyOs::new(mem);
    ///
    /// // Create a process with 1+10 randomly placed regions
    /// let pid = os.alloc_process(size::kb(4), &[]);
    /// let proc = os.process_by_pid(pid).unwrap().proc;
    /// os.process_alloc_random_mem(&proc, 10, 1);
    /// let mut mem = os.process_by_pid(pid).unwrap().mem;
    ///
    /// // Translate entire address space
    /// let mut output = vec![];
    ///
    /// mem.virt_translation_map_range(
    ///     Address::null(),
    ///     Address::invalid(),
    ///     (&mut output).into()
    /// );
    ///
    /// // There should be 11 memory ranges.
    /// assert_eq!(11, output.len());
    /// ```
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

    /// Retrieves mapped virtual pages in the specified range.
    ///
    /// In case a range from [`Address::null()`], [`Address::invalid()`] is specified
    /// this function will return all mappings.
    ///
    /// Given negative gap size, they will not be removed.
    ///
    /// # Example:
    ///
    /// ```
    /// use memflow::prelude::v1::*;
    /// # use memflow::dummy::{DummyMemory, DummyOs};
    /// # use memflow::architecture::x86::x64;
    /// # let dummy_mem = DummyMemory::new(size::mb(16));
    /// # let mut dummy_os = DummyOs::new(dummy_mem);
    /// # let (dtb, virt_base) = dummy_os.alloc_dtb(size::mb(2), &[]);
    /// # let translator = x64::new_translator(dtb);
    /// # let arch = x64::ARCH;
    /// # let mut virt_mem = VirtualDma::new(dummy_os.forward_mut(), arch, translator);
    /// println!("{:>16} {:>12} {:<}", "ADDR", "SIZE", "TYPE");
    ///
    /// let callback = &mut |CTup3(addr, size, pagety)| {
    ///     println!("{:>16x} {:>12x} {:<?}", addr, size, pagety);
    ///     true
    /// };
    ///
    /// // display all mappings with a gap size of 0
    /// virt_mem.virt_page_map_range(0, Address::null(), Address::invalid(), callback.into());
    /// ```
    fn virt_page_map_range(
        &mut self,
        gap_size: imem,
        start: Address,
        end: Address,
        out: MemoryRangeCallback,
    ) {
        let mut gap_remover = GapRemover::new(out, gap_size, start, end);

        self.virt_to_phys_range(
            start,
            end,
            (&mut |VirtualTranslation {
                       in_virtual,
                       size,
                       out_physical,
                   }| {
                gap_remover.push_range(CTup3(in_virtual, size, out_physical.page_type));
                true
            })
                .into(),
        );
    }

    /// Translate a single virtual address into a single physical address.
    ///
    /// This is the simplest translation function that performs single address translation.
    ///
    /// # Example:
    ///
    /// ```
    /// use memflow::prelude::v1::*;
    /// # use memflow::dummy::DummyOs;
    ///
    /// // Virtual translation test
    /// fn vtop(mem: &mut impl VirtualTranslate, addr: Address) {
    ///     assert!(mem.virt_to_phys(addr).is_ok());
    /// }
    /// # let mut proc = DummyOs::quick_process(size::mb(2), &[]);
    /// # let addr = proc.info().address;
    /// # vtop(&mut proc.mem, addr);
    /// ```
    fn virt_to_phys(&mut self, address: Address) -> Result<PhysicalAddress> {
        let mut out = Err(Error(ErrorOrigin::VirtualTranslate, ErrorKind::OutOfBounds));

        self.virt_to_phys_list(
            &[CTup2(address, 1)],
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

    /// Retrieve page information at virtual address.
    ///
    /// This function is equivalent to calling
    /// [containing_page](crate::types::physical_address::PhysicalAddress::containing_page) on
    /// [`virt_to_phys`](Self::virt_to_phys) result.
    ///
    /// # Example:
    ///
    /// ```
    /// use memflow::prelude::v1::*;
    /// # use memflow::dummy::DummyOs;
    ///
    /// // Virtual translation test
    /// fn vtop(mem: &mut impl VirtualTranslate, addr: Address) {
    ///     let page = mem.virt_page_info(addr).unwrap();
    ///     assert_eq!(page.page_size, mem::kb(4));
    ///     assert_eq!(page.page_type, PageType::WRITEABLE);
    /// }
    /// # let mut proc = DummyOs::quick_process(size::mb(2), &[]);
    /// # let addr = proc.info().address;
    /// # vtop(&mut proc.mem, addr);
    /// ```
    fn virt_page_info(&mut self, addr: Address) -> Result<Page> {
        let paddr = self.virt_to_phys(addr)?;
        Ok(paddr.containing_page())
    }

    /// Retrieve a vector of physical pages within given range.
    ///
    /// This is equivalent to calling [`virt_page_map_range`](Self::virt_page_map_range) with a
    /// vector output argument.
    ///
    /// # Example:
    ///
    /// ```
    /// use memflow::prelude::v1::*;
    /// # use memflow::dummy::{DummyMemory, DummyOs};
    /// # use memflow::architecture::x86::x64;
    /// # let dummy_mem = DummyMemory::new(size::mb(16));
    /// # let mut dummy_os = DummyOs::new(dummy_mem);
    /// # let (dtb, virt_base) = dummy_os.alloc_dtb(size::mb(2), &[]);
    /// # let translator = x64::new_translator(dtb);
    /// # let arch = x64::ARCH;
    /// # let mut virt_mem = VirtualDma::new(dummy_os.forward_mut(), arch, translator);
    /// println!("{:>16} {:>12} {:<}", "ADDR", "SIZE", "TYPE");
    ///
    /// // display all mappings with a gap size of 0
    /// let out = virt_mem.virt_page_map_range_vec(0, Address::null(), Address::invalid());
    ///
    /// assert!(out.len() > 0);
    ///
    /// for CTup3(addr, size, pagety) in out {
    ///     println!("{:>16x} {:>12x} {:<?}", addr, size, pagety);
    /// }
    /// ```
    #[skip_func]
    fn virt_page_map_range_vec(
        &mut self,
        gap_size: imem,
        start: Address,
        end: Address,
    ) -> Vec<MemoryRange> {
        let mut out = vec![];
        self.virt_page_map_range(gap_size, start, end, (&mut out).into());
        out
    }

    // page map helpers

    /// Get virtual translation map over entire address space.
    ///
    /// This is equivalent to [`virt_translation_map_range`](Self::virt_translation_map_range) with a
    /// range from null to highest address.
    ///
    /// # Example:
    ///
    /// ```
    /// use memflow::prelude::v1::*;
    /// use memflow::dummy::{DummyOs, DummyMemory};
    ///
    /// // Create a dummy OS
    /// let mem = DummyMemory::new(size::mb(1));
    /// let mut os = DummyOs::new(mem);
    ///
    /// // Create a process with 1+10 randomly placed regions
    /// let pid = os.alloc_process(size::kb(4), &[]);
    /// let proc = os.process_by_pid(pid).unwrap().proc;
    /// os.process_alloc_random_mem(&proc, 10, 1);
    /// let mut mem = os.process_by_pid(pid).unwrap().mem;
    ///
    /// // Translate entire address space
    /// let mut output = vec![];
    ///
    /// mem.virt_translation_map((&mut output).into());
    ///
    /// // There should be 11 memory ranges.
    /// assert_eq!(11, output.len());
    /// ```
    fn virt_translation_map(&mut self, out: VirtualTranslationCallback) {
        self.virt_translation_map_range(Address::null(), Address::invalid(), out)
    }

    /// Get virtual translation map over entire address space and return it as a vector.
    ///
    /// This is a [`virt_translation_map`](Self::virt_translation_map) helper that stores results
    /// into a vector that gets returned.
    ///
    /// # Example:
    ///
    /// ```
    /// use memflow::prelude::v1::*;
    /// use memflow::dummy::{DummyOs, DummyMemory};
    ///
    /// // Create a dummy OS
    /// let mem = DummyMemory::new(size::mb(1));
    /// let mut os = DummyOs::new(mem);
    ///
    /// // Create a process with 1+10 randomly placed regions
    /// let pid = os.alloc_process(size::kb(4), &[]);
    /// let proc = os.process_by_pid(pid).unwrap().proc;
    /// os.process_alloc_random_mem(&proc, 10, 1);
    /// let mut mem = os.process_by_pid(pid).unwrap().mem;
    ///
    /// // Translate entire address space
    /// let output = mem.virt_translation_map_vec();
    ///
    /// // There should be 11 memory ranges.
    /// assert_eq!(11, output.len());
    /// ```
    #[skip_func]
    fn virt_translation_map_vec(&mut self) -> Vec<VirtualTranslation> {
        let mut out = vec![];
        self.virt_translation_map((&mut out).into());
        out
    }

    /// Attempt to translate a physical address into a virtual one.
    ///
    /// This function is the reverse of [`virt_to_phys`](Self::virt_to_phys). Note, that there
    /// could could be multiple virtual addresses for one physical address. If all candidates
    /// are needed, use [`phys_to_virt_vec`](Self::phys_to_virt_vec) function.
    ///
    /// # Example:
    ///
    /// ```
    /// use memflow::prelude::v1::*;
    /// # use memflow::dummy::DummyOs;
    ///
    /// // Virtual translation and reversal test
    /// fn vtop_ptov(mem: &mut impl VirtualTranslate, addr: Address) {
    ///     let paddr = mem.virt_to_phys(addr).unwrap();
    ///     let vaddr = mem.phys_to_virt(paddr.address());
    ///     assert_eq!(vaddr, Some(addr));
    /// }
    /// # let mut proc = DummyOs::quick_process(size::mb(2), &[]);
    /// # let addr = proc.info().address;
    /// # vtop_ptov(&mut proc.mem, addr);
    /// ```
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
    ///
    /// This function is the reverse of [`virt_to_phys`](Self::virt_to_phys), and it retrieves all
    /// physical addresses that map to this virtual address.
    ///
    /// # Example:
    ///
    /// ```
    /// use memflow::prelude::v1::*;
    /// # use memflow::dummy::DummyOs;
    ///
    /// // Virtual translation and reversal test
    /// fn vtop_ptov(mem: &mut impl VirtualTranslate, addr: Address) {
    ///     let paddr = mem.virt_to_phys(addr).unwrap();
    ///     let vaddrs = mem.phys_to_virt_vec(paddr.address());
    ///     assert_eq!(&vaddrs, &[addr]);
    /// }
    /// # let mut proc = DummyOs::quick_process(size::mb(2), &[]);
    /// # let addr = proc.info().address;
    /// # vtop_ptov(&mut proc.mem, addr);
    /// ```
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

    /// Retrieves all mapped virtual pages.
    ///
    /// The [`virt_page_map`](Self::virt_page_map) function is a convenience wrapper for calling
    /// [`virt_page_map_range`](Self::virt_page_map_range)`(gap_size, Address::null(), Address::invalid(), out)`.
    ///
    /// # Example:
    ///
    /// ```
    /// use memflow::prelude::v1::*;
    /// # use memflow::dummy::{DummyMemory, DummyOs};
    /// # use memflow::architecture::x86::x64;
    /// # let dummy_mem = DummyMemory::new(size::mb(16));
    /// # let mut dummy_os = DummyOs::new(dummy_mem);
    /// # let (dtb, virt_base) = dummy_os.alloc_dtb(size::mb(2), &[]);
    /// # let translator = x64::new_translator(dtb);
    /// # let arch = x64::ARCH;
    /// # let mut virt_mem = VirtualDma::new(dummy_os.forward_mut(), arch, translator);
    /// println!("{:>16} {:>12} {:<}", "ADDR", "SIZE", "TYPE");
    ///
    /// let callback = &mut |CTup3(addr, size, pagety)| {
    ///     println!("{:>16x} {:>12x} {:<?}", addr, size, pagety);
    ///     true
    /// };
    ///
    /// // display all mappings with a gap size of 0
    /// virt_mem.virt_page_map(0, callback.into());
    /// ```
    fn virt_page_map(&mut self, gap_size: imem, out: MemoryRangeCallback) {
        self.virt_page_map_range(gap_size, Address::null(), Address::invalid(), out)
    }

    /// Returns a [`Vec`] of all mapped virtual pages.
    ///
    /// The [`virt_page_map`](Self::virt_page_map) function is a convenience wrapper for calling
    /// [`virt_page_map_range`](Self::virt_page_map_range)`(gap_size, Address::null(), Address::invalid(), out)`.
    ///
    /// # Remarks:
    ///
    /// This function has to allocate all MemoryRanges when they are put into a [`Vec`].
    /// If the additional allocations are undesired please use the provided [`virt_page_map`](Self::virt_page_map) with an appropiate callback.
    ///
    /// # Example:
    ///
    /// ```
    /// use memflow::prelude::v1::*;
    /// # use memflow::dummy::{DummyMemory, DummyOs};
    /// # use memflow::architecture::x86::x64;
    /// # let dummy_mem = DummyMemory::new(size::mb(16));
    /// # let mut dummy_os = DummyOs::new(dummy_mem);
    /// # let (dtb, virt_base) = dummy_os.alloc_dtb(size::mb(2), &[]);
    /// # let translator = x64::new_translator(dtb);
    /// # let arch = x64::ARCH;
    /// # let mut virt_mem = VirtualDma::new(dummy_os.forward_mut(), arch, translator);
    /// // acquire all mappings with a gap size of 0
    /// let maps = virt_mem.virt_page_map_vec(0);
    ///
    /// println!("{:>16} {:>12} {:<}", "ADDR", "SIZE", "TYPE");
    /// for CTup3(addr, size, pagety) in maps.iter() {
    ///     println!("{:>16x} {:>12x} {:<?}", addr, size, pagety);
    /// };
    /// ```
    #[skip_func]
    fn virt_page_map_vec(&mut self, gap_size: imem) -> Vec<MemoryRange> {
        let mut out = vec![];
        self.virt_page_map(gap_size, (&mut out).into());
        out
    }
}

pub type VirtualTranslationCallback<'a> = OpaqueCallback<'a, VirtualTranslation>;
pub type VirtualTranslationFailCallback<'a> = OpaqueCallback<'a, VirtualTranslationFail>;

/// Virtual page range information with physical mappings used for callbacks
#[repr(C)]
#[derive(Clone, Debug, Eq, Copy)]
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[cfg_attr(feature = "abi_stable", derive(::abi_stable::StableAbi))]
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

#[repr(C)]
#[derive(Clone, Copy, Debug)]
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[cfg_attr(feature = "abi_stable", derive(::abi_stable::StableAbi))]
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
    /// # use memflow::types::{PhysicalAddress, Address, umem};
    /// # use memflow::dummy::{DummyMemory, DummyOs};
    /// use memflow::mem::{VirtualTranslate2, DirectTranslate};
    /// use memflow::types::size;
    /// use memflow::architecture::x86::x64;
    /// use memflow::cglue::{FromExtend, CTup3};
    ///
    /// use std::convert::TryInto;
    ///
    /// # const VIRT_MEM_SIZE: usize = size::mb(8) as usize;
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
    ///     .map(|(i, buf)| CTup3(virtual_base + ((i + 1) * size::kb(4) - 1), Address::NULL, buf));
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
        VI: Iterator<Item = CTup3<Address, Address, B>>;

    /// Translate a single virtual address
    ///
    /// This function will do a virtual to physical memory translation for the
    /// `VirtualTranslate3` for single address returning either PhysicalAddress, or an error.
    ///
    /// # Examples
    /// ```
    /// # use memflow::error::Result;
    /// # use memflow::types::{PhysicalAddress, Address, umem};
    /// # use memflow::dummy::{DummyMemory, DummyOs};
    /// # use memflow::types::size;
    /// # use memflow::mem::VirtualTranslate3;
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
        let success = &mut |elem: CTup3<PhysicalAddress, Address, _>| {
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
            Some(CTup3::<_, _, umem>(vaddr, vaddr, 1)).into_iter(),
            &mut success.into(),
            &mut fail.into(),
        );
        output.map(Ok).unwrap_or_else(|| Err(output_err.unwrap()))
    }
}

// forward impls
impl<T, P> VirtualTranslate2 for P
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
        VI: Iterator<Item = CTup3<Address, Address, B>>,
    {
        (**self).virt_to_phys_iter(phys_mem, translator, addrs, out, out_fail)
    }
}

/// Translates virtual memory to physical using internal translation base (usually a process' dtb)
///
/// This trait abstracts virtual address translation for a single virtual memory scope.
/// On x86 architectures, it is a single `Address` - a CR3 register. But other architectures may
/// use multiple translation bases, or use a completely different translation mechanism (MIPS).
pub trait VirtualTranslate3: Clone + Copy + Send {
    /// Translate a single virtual address
    ///
    /// # Examples
    /// ```
    /// # use memflow::error::Result;
    /// # use memflow::types::{PhysicalAddress, Address};
    /// # use memflow::dummy::{DummyMemory, DummyOs};
    /// use memflow::mem::VirtualTranslate3;
    /// use memflow::architecture::x86::x64;
    /// use memflow::types::{size, umem};
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
    /// // Translate a mapped address
    /// let res = translator.virt_to_phys(
    ///     &mut mem,
    ///     virtual_base,
    /// );
    ///
    /// assert!(res.is_ok());
    ///
    /// // Translate unmapped address
    /// let res = translator.virt_to_phys(
    ///     &mut mem,
    ///     virtual_base - 1,
    /// );
    ///
    /// assert!(res.is_err());
    ///
    /// ```
    fn virt_to_phys<T: PhysicalMemory>(
        &self,
        mem: &mut T,
        addr: Address,
    ) -> Result<PhysicalAddress> {
        let mut buf: [std::mem::MaybeUninit<u8>; 512] =
            unsafe { std::mem::MaybeUninit::uninit().assume_init() };
        let mut output = None;
        let success = &mut |elem: CTup3<PhysicalAddress, Address, _>| {
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
            mem,
            Some(CTup3::<_, _, umem>(addr, addr, 1)).into_iter(),
            &mut success.into(),
            &mut fail.into(),
            &mut buf,
        );
        output.map(Ok).unwrap_or_else(|| Err(output_err.unwrap()))
    }

    fn virt_to_phys_iter<
        T: PhysicalMemory + ?Sized,
        B: SplitAtIndex,
        VI: Iterator<Item = CTup3<Address, Address, B>>,
    >(
        &self,
        mem: &mut T,
        addrs: VI,
        out: &mut VtopOutputCallback<B>,
        out_fail: &mut VtopFailureCallback<B>,
        tmp_buf: &mut [std::mem::MaybeUninit<u8>],
    );

    fn translation_table_id(&self, address: Address) -> umem;

    fn arch(&self) -> ArchitectureObj;
}

pub type VtopOutputCallback<'a, B> = OpaqueCallback<'a, CTup3<PhysicalAddress, Address, B>>;
pub type VtopFailureCallback<'a, B> = OpaqueCallback<'a, (Error, CTup3<Address, Address, B>)>;
