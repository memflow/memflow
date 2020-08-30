pub(crate) mod translate_data;

use crate::error::{Error, Result};
use crate::iter::{PageChunks, SplitAtIndex};
use crate::mem::{PhysicalMemory, PhysicalReadData};
use crate::types::{Address, PageType, PhysicalAddress};
use std::convert::TryInto;
use translate_data::{TranslateData, TranslateVec, TranslationChunk};

use bumpalo::{collections::Vec as BumpVec, Bump};
use vector_trees::{BVecTreeMap as BTreeMap, Vector};

#[cfg(feature = "trace_mmu")]
macro_rules! vtop_trace {
    ( $( $x:expr ),* ) => {
        log::trace!( $($x, )* );
    }
}

#[cfg(not(feature = "trace_mmu"))]
macro_rules! vtop_trace {
    ( $( $x:expr ),* ) => {};
}

/// The `ArchMMUSpec` structure defines how a real memory management unit should behave when
/// translating virtual memory addresses to physical ones.
///
/// The core logic of virtual to physical memory translation is practically the same, but different
/// MMUs may have different address space sizes, and thus split the addresses in different ways.
///
/// For instance, most x86_64 architectures have 4 levels of page mapping, providing 52-bit address
/// space. Virtual address gets split into 4 9-bit regions, and a 12-bit one, the first 4 are used
/// to index the page tables, and the last, 12-bit split is used as an offset to get the final
/// memory address. Meanwhile, x86 with PAE has 3 levels of page mapping, providing 36-bit address
/// space. Virtual address gets split into a 2-bit, 2 9-bit and a 12-bit regions - the last one is
/// also used as an offset from the physical frame. The difference is of level count, and virtual
/// address splits, but the core page table walk stays the same.
///
/// Our virtual to physical memory ranslation code is the same for both architectures, in fact, it
/// is also the same for the x86 (non-PAE) architecture that has different PTE and pointer sizes.
/// All that differentiates the translation process is the data inside this structure.
#[derive(Debug)]
pub struct ArchMMUSpec {
    /// defines the way virtual addresses gets split (the last element
    /// being the final physical page offset, and thus treated a bit differently)
    pub virtual_address_splits: &'static [u8],
    /// defines at which page mapping steps we can return a large page.
    /// Steps are indexed from 0, and the list has to be sorted, otherwise the code may fail.
    pub valid_final_page_steps: &'static [usize],
    /// define the address space upper bound (32 for x86, 52 for x86_64)
    pub address_space_bits: u8,
    /// native pointer size in bytes for the architecture.
    pub addr_size: u8,
    /// size of an individual page table entry in bytes.
    pub pte_size: usize,
    /// index of a bit in PTE defining whether the page is present or not.
    pub present_bit: u8,
    /// index of a bit in PTE defining if the page is writeable.
    pub writeable_bit: u8,
    /// index of a bit in PTE defining if the page is non-executable.
    pub nx_bit: u8,
    /// index of a bit in PTE defining if the PTE points to a large page.
    pub large_page_bit: u8,
}

pub trait MMUTranslationBase {
    fn get_initial_pt(&self, address: Address) -> Address;

    fn get_pt_by_index(&self, _: usize) -> Address;

    fn pt_count(&self) -> usize;

    fn virt_addr_filter<B: SplitAtIndex, O: Extend<(Error, Address, B)>>(
        &self,
        spec: &ArchMMUSpec,
        addr: (Address, B),
        data_to_translate: &mut TranslateVec<B>,
        out_fail: &mut O,
    );
}

impl ArchMMUSpec {
    /// Mask a page table entry address to retrieve the next page table entry
    ///
    /// This function uses virtual_address_splits to mask the first bits out in `pte_addr`, but
    /// keep everything else until the `address_space_bits` upper bound.
    ///
    /// # Arguments
    ///
    /// * `pte_addr` - page table entry address to mask
    /// * `step` - the current step in the page walk
    ///
    /// # Remarks
    ///
    /// The final step is handled differently, because the final split provides a byte offset to
    /// the page, instead of an offset that has to be multiplied by `pte_size`. We do that by
    /// subtracting `pte_size` logarithm from the split size.
    pub fn pte_addr_mask(&self, pte_addr: Address, step: usize) -> u64 {
        let max = self.address_space_bits - 1;
        let min = self.virtual_address_splits[step]
            + if step == self.virtual_address_splits.len() - 1 {
                0
            } else {
                self.pte_size.to_le().trailing_zeros() as u8
            };
        let mask = Address::bit_mask(min..max);
        vtop_trace!("pte_addr_mask={:b}", mask.as_u64());
        pte_addr.as_u64() & mask.as_u64()
    }

    /// Filter out the input virtual address range to be in bounds
    ///
    ///
    /// # Arguments
    ///
    /// * `(addr, buf)` - an address and buffer pair that gets split and filtered
    /// * `valid_out` - output collection that contains valid splits
    /// * `fail_out` - the final collection where the function will push rejected ranges to
    ///
    /// # Remarks
    ///
    /// This function cuts the input virtual address to be inside range `(-2^address_space_bits;
    /// +2^address_space_bits)`. It may result in 2 ranges, and it may have up to 2 failed ranges
    pub(crate) fn virt_addr_filter<B, VO, FO>(
        &self,
        (addr, buf): (Address, B),
        valid_out: &mut VO,
        fail_out: &mut FO,
    ) where
        B: SplitAtIndex,
        VO: Extend<TranslateData<B>>,
        FO: Extend<(Error, Address, B)>,
    {
        let mut tr_data = TranslateData { addr, buf };

        let (mut left, reject) =
            tr_data.split_inclusive_at(Address::bit_mask(0..(self.addr_size * 8 - 1)).as_usize());

        if let Some(data) = reject {
            fail_out.extend(Some((Error::VirtualTranslate, data.addr, data.buf)));
        }

        let virt_bit_range = self.virt_addr_bit_range(0).1;
        let virt_range = 1usize << (virt_bit_range - 1);

        let (lower, higher) = left.split_at(virt_range);

        if lower.length() > 0 {
            valid_out.extend(Some(lower).into_iter());
        }

        if let Some(mut data) = higher {
            let (reject, higher) = data.split_at_rev(virt_range);

            // The upper half has to be all negative (all bits set), so compare the masks to see if
            // it is the case.
            let lhs = Address::bit_mask(virt_bit_range..(self.addr_size * 8 - 1)).as_u64();
            let rhs = higher.addr.as_u64() & lhs;

            if (lhs ^ rhs) != 0 {
                return;
            }

            if higher.length() > 0 {
                valid_out.extend(Some(higher).into_iter());
            }

            if let Some(data) = reject {
                fail_out.extend(Some((Error::VirtualTranslate, data.addr, data.buf)));
            }
        }
    }

    fn virt_addr_bit_range(&self, step: usize) -> (u8, u8) {
        let max_index_bits = self.virtual_address_splits[step..].iter().sum::<u8>();
        let min_index_bits = max_index_bits - self.virtual_address_splits[step];
        (min_index_bits, max_index_bits)
    }

    fn virt_addr_to_pte_offset(&self, virt_addr: Address, step: usize) -> u64 {
        let (min, max) = self.virt_addr_bit_range(step);
        vtop_trace!("virt_addr_bit_range for step {} = ({}, {})", step, min, max);

        let shifted = virt_addr.as_u64() >> min;
        let mask = Address::bit_mask(0..(max - min - 1));

        (shifted & mask.as_u64()) * self.pte_size as u64
    }

    fn virt_addr_to_page_offset(&self, virt_addr: Address, step: usize) -> u64 {
        let max = self.virt_addr_bit_range(step).1;
        virt_addr.as_u64() & Address::bit_mask(0..(max - 1)).as_u64()
    }

    /// Return the number of splits of virtual addresses
    ///
    /// The returned value will be one more than the number of page table levels
    pub fn split_count(&self) -> usize {
        self.virtual_address_splits.len()
    }

    /// Calculate the size of the page table entry leaf in bytes
    ///
    /// This will return the number of page table entries at a specific step multiplied by the
    /// `pte_size`. Usually this will be an entire page, but in certain cases, like the highest
    /// mapping level of x86 with PAE, it will be less.
    ///
    /// # Arguments
    ///
    /// * `step` - the current step in the page walk
    pub fn pt_leaf_size(&self, step: usize) -> usize {
        let (min, max) = self.virt_addr_bit_range(step);
        (1 << (max - min)) * self.pte_size
    }

    /// Perform a virtual translation step, returning the next PTE address to read
    ///
    /// # Arguments
    ///
    /// * `pte_addr` - input PTE address that was read the last time (or DTB)
    /// * `virt_addr` - virtual address we are translating
    /// * `step` - the current step in the page walk
    pub fn vtop_step(&self, pte_addr: Address, virt_addr: Address, step: usize) -> Address {
        Address::from(
            self.pte_addr_mask(pte_addr, step) | self.virt_addr_to_pte_offset(virt_addr, step),
        )
    }

    /// Get the page size of a specific step without checking if such page could exist
    ///
    /// # Arguments
    ///
    /// * `step` - the current step in the page walk
    pub fn page_size_step_unchecked(&self, step: usize) -> usize {
        let max_index_bits = self.virtual_address_splits[step..].iter().sum::<u8>();
        (1u64 << max_index_bits) as usize
    }

    /// Get the page size of a specific page walk step
    ///
    /// This function is preferable to use externally, because in debug builds it will check if such
    /// page could exist, and if can not, it will panic
    ///
    /// # Arguments
    ///
    /// * `step` - the current step in the page walk
    pub fn page_size_step(&self, step: usize) -> usize {
        debug_assert!(self.valid_final_page_steps.binary_search(&step).is_ok());
        self.page_size_step_unchecked(step)
    }

    /// Get the page size of a specific mapping level
    ///
    /// This function is the same as `page_size_step`, but the `level` almost gets inverted. It
    /// goes in line with x86 page level naming. With 1 being the 4kb page, and higher meaning
    /// larger page.
    ///
    /// # Arguments
    ///
    /// * `level` - page mapping level to get the size of (1 meaning the smallest page)
    pub fn page_size_level(&self, level: usize) -> usize {
        self.page_size_step(self.virtual_address_splits.len() - level)
    }

    /// Get the final physical page
    ///
    /// This performs the final step of a successful translation - retrieve the final physical
    /// address. It does not perform any present checks, and assumes `pte_addr` points to a valid
    /// page.
    ///
    /// # Arguments
    ///
    /// * `pte_addr` - the address inside the previously read PTE
    /// * `virt_addr` - the virtual address we are currently translating
    /// * `step` - the current step in the page walk
    pub fn get_phys_page(
        &self,
        pte_addr: Address,
        virt_addr: Address,
        step: usize,
    ) -> PhysicalAddress {
        let phys_addr = Address::from(
            self.pte_addr_mask(pte_addr, step) | self.virt_addr_to_page_offset(virt_addr, step),
        );

        PhysicalAddress::with_page(
            phys_addr,
            PageType::default()
                .write(pte_addr.bit_at(self.writeable_bit))
                .noexec(pte_addr.bit_at(self.nx_bit)),
            self.page_size_step(step),
        )
    }

    /// Check if the current page table entry is valid
    ///
    /// # Arguments
    ///
    /// * `pte_addr` - current page table entry
    /// * `step` - the current step in the page walk
    pub fn check_entry(&self, pte_addr: Address, step: usize) -> bool {
        step == 0 || pte_addr.bit_at(self.present_bit)
    }

    /// Check if the current page table entry contains a physical page
    ///
    /// This will check `valid_final_page_steps` to determine if the PTE could have a large page,
    /// and then check the large page bit for confirmation. It will always return true on the final
    /// mapping regarding of the values in `valid_final_page_steps`. The `valid_final_page_steps`
    /// list has to be sorted for the function to work properly, because it uses binary search.
    ///
    /// # Arguments
    ///
    /// * `pte_addr` - current page table entry
    /// * `step` - the current step the page walk
    pub fn is_final_mapping(&self, pte_addr: Address, step: usize) -> bool {
        (step == self.virtual_address_splits.len() - 1)
            || (pte_addr.bit_at(self.large_page_bit)
                && self.valid_final_page_steps.binary_search(&step).is_ok())
    }

    /// This function will do a virtual to physical memory translation for the `ArchMMUSpec` in
    /// `MMUTranslationBase` scope, over multiple elements.
    pub(crate) fn virt_to_phys_iter<T, B, D, VI, VO, FO>(
        &self,
        mem: &mut T,
        dtb: D,
        addrs: VI,
        out: &mut VO,
        out_fail: &mut FO,
        arena: &Bump,
    ) where
        T: PhysicalMemory + ?Sized,
        B: SplitAtIndex,
        D: MMUTranslationBase,
        VI: Iterator<Item = (Address, B)>,
        VO: Extend<(PhysicalAddress, B)>,
        FO: Extend<(Error, Address, B)>,
    {
        vtop_trace!("virt_to_phys_iter_with_mmu");

        let mut data_to_translate = BumpVec::new_in(arena);
        let mut data_pt_read: BumpVec<PhysicalReadData> = BumpVec::new_in(arena);
        let mut data_pt_buf = BumpVec::new_in(arena);
        let mut data_to_translate_map = BTreeMap::new_in(BumpVec::new_in(arena));

        //TODO: Calculate and reserve enough data in the data_to_translate vectors
        //TODO: precalc vtop_step bit split sum / transform the splits to a lookup table
        //TODO: Improve filtering speed (vec reserve)
        //TODO: Optimize BTreeMap

        data_to_translate
            .extend((0..dtb.pt_count()).map(|idx| {
                TranslationChunk::new(dtb.get_pt_by_index(idx), BumpVec::new_in(arena))
            }));

        addrs.for_each(|data| dtb.virt_addr_filter(self, data, &mut data_to_translate, out_fail));

        data_to_translate
            .iter_mut()
            .for_each(|trd| trd.recalc_minmax());

        for pt_step in 0..self.split_count() {
            vtop_trace!(
                "pt_step = {}, data_to_translate.len() = {:x}",
                pt_step,
                data_to_translate.len()
            );

            let next_page_size = self.page_size_step_unchecked(pt_step + 1);

            vtop_trace!("next_page_size = {:x}", next_page_size);

            //Loop through the data in reverse order to allow the data buffer grow on the back when
            //memory regions are split
            for i in (0..data_to_translate.len()).rev() {
                let tr_chunk = data_to_translate.swap_remove(i);
                vtop_trace!(
                    "checking pt_addr={:x}, elems={:x}",
                    tr_chunk.pt_addr,
                    tr_chunk.vec.len()
                );

                if !self.check_entry(tr_chunk.pt_addr, pt_step) {
                    //There has been an error in translation, push it to output with the associated buf
                    vtop_trace!("check_entry failed");
                    out_fail.extend(
                        tr_chunk
                            .vec
                            .into_iter()
                            .map(|entry| (Error::VirtualTranslate, entry.addr, entry.buf)),
                    );
                } else if self.is_final_mapping(tr_chunk.pt_addr, pt_step) {
                    //We reached an actual page. The translation was successful
                    vtop_trace!("found final mapping: {:x}", tr_chunk.pt_addr);
                    let pt_addr = tr_chunk.pt_addr;
                    out.extend(tr_chunk.vec.into_iter().map(|entry| {
                        (self.get_phys_page(pt_addr, entry.addr, pt_step), entry.buf)
                    }));
                } else {
                    //We still need to continue the page walk

                    let min_addr = tr_chunk.min_addr();

                    //As an optimization, divide and conquer the input memory regions.
                    //VTOP speedup is insane. Visible in large sequential or chunked reads.
                    for (_, (_, mut chunk)) in
                        (arena, tr_chunk).page_chunks(min_addr, next_page_size)
                    {
                        let pt_addr = self.vtop_step(chunk.pt_addr, chunk.min_addr(), pt_step);
                        chunk.pt_addr = pt_addr;
                        data_to_translate.push(chunk);
                    }
                }
            }

            if data_to_translate.is_empty() {
                break;
            }

            if let Err(err) = self.read_pt_address_iter(
                mem,
                pt_step,
                &mut data_to_translate_map,
                &mut data_to_translate,
                &mut data_pt_buf,
                &mut data_pt_read,
                out_fail,
            ) {
                vtop_trace!("read_pt_address_iter failure: {}", err);
                out_fail.extend(
                    data_to_translate
                        .into_iter()
                        .flat_map(|chunk| chunk.vec.into_iter())
                        .map(|data| (err, data.addr, data.buf)),
                );
                return;
            }
        }

        debug_assert!(data_to_translate.is_empty());
    }

    //TODO: Clean this up to have less args
    #[allow(clippy::too_many_arguments)]
    fn read_pt_address_iter<'a, T, B, V, FO>(
        &self,
        mem: &mut T,
        step: usize,
        addr_map: &mut BTreeMap<V, Address, ()>,
        addrs: &mut TranslateVec<'a, B>,
        pt_buf: &mut BumpVec<u8>,
        pt_read: &mut BumpVec<PhysicalReadData>,
        err_out: &mut FO,
    ) -> Result<()>
    where
        T: PhysicalMemory + ?Sized,
        FO: Extend<(Error, Address, B)>,
        V: Vector<vector_trees::btree::BVecTreeNode<Address, ()>>,
        B: SplitAtIndex,
    {
        //TODO: use self.pt_leaf_size(step) (need to handle LittleEndian::read_u64)
        let pte_size = 8;
        let page_size = self.pt_leaf_size(step);

        //pt_buf.clear();
        pt_buf.resize(pte_size * addrs.len(), 0);

        debug_assert!(pt_read.is_empty());

        //This is safe, because pt_read gets cleared at the end of the function
        let pt_read: &mut BumpVec<PhysicalReadData> = unsafe { std::mem::transmute(pt_read) };

        for (chunk, tr_chunk) in pt_buf.chunks_exact_mut(pte_size).zip(addrs.iter()) {
            pt_read.push(PhysicalReadData(
                PhysicalAddress::with_page(tr_chunk.pt_addr, PageType::PAGE_TABLE, page_size),
                chunk,
            ));
        }

        mem.phys_read_raw_list(pt_read)?;

        //Filter out duplicate reads
        //Ideally, we would want to append all duplicates to the existing list, but they would mostly
        //only occur, in strange kernel side situations when building the page map,
        //and having such handling may end up highly inefficient (due to having to use map, and remapping it)
        addr_map.clear();

        //Okay, so this is extremely useful in one element reads.
        //We kind of have a local on-stack cache to check against
        //before a) checking in the set, and b) pushing to the set
        let mut prev_addr: Option<Address> = None;

        for i in (0..addrs.len()).rev() {
            let mut chunk = addrs.swap_remove(i);
            let PhysicalReadData(_, buf) = pt_read.swap_remove(i);
            let pt_addr = Address::from(u64::from_le_bytes(buf[0..8].try_into().unwrap()));

            if self.pte_addr_mask(chunk.pt_addr, step) != self.pte_addr_mask(pt_addr, step)
                && (prev_addr.is_none()
                    || (prev_addr.unwrap() != pt_addr && !addr_map.contains_key(&pt_addr)))
            {
                chunk.pt_addr = pt_addr;

                if let Some(pa) = prev_addr {
                    addr_map.insert(pa, ());
                }

                prev_addr = Some(pt_addr);
                addrs.push(chunk);
                continue;
            }

            err_out.extend(
                chunk
                    .vec
                    .into_iter()
                    .map(|entry| (Error::VirtualTranslate, entry.addr, entry.buf)),
            );
        }

        pt_read.clear();

        Ok(())
    }
}
