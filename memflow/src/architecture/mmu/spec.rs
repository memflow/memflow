use super::ArchMMUDef;
use crate::architecture::Endianess;

use super::translate_data::{TranslateData, TranslateDataVec, TranslateVec, TranslationChunk};
use super::MMUTranslationBase;
use crate::error::{Error, Result};
use crate::iter::SplitAtIndex;
use crate::mem::{PhysicalMemory, PhysicalReadData};
use crate::types::{Address, PageType, PhysicalAddress};
pub(crate) use fixed_slice_vec::FixedSliceVec as MVec;
use std::convert::TryInto;

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

const MAX_LEVELS: usize = 8;

pub struct ArchMMUSpec {
    pub def: ArchMMUDef,
    pub pte_addr_masks: [u64; MAX_LEVELS],
    pub virt_addr_bit_ranges: [(u8, u8); MAX_LEVELS],
    pub virt_addr_masks: [u64; MAX_LEVELS],
    pub virt_addr_page_masks: [u64; MAX_LEVELS],
    pub valid_final_page_steps: [bool; MAX_LEVELS],
    pub pt_leaf_size: [usize; MAX_LEVELS],
    pub page_size_step: [usize; MAX_LEVELS],
    pub spare_allocs: usize,
}

impl From<ArchMMUDef> for ArchMMUSpec {
    fn from(def: ArchMMUDef) -> Self {
        Self::from_def(def)
    }
}

impl ArchMMUSpec {
    pub const fn from_def(def: ArchMMUDef) -> Self {
        let mut pte_addr_masks = [0; MAX_LEVELS];
        let mut virt_addr_bit_ranges = [(0, 0); MAX_LEVELS];
        let mut virt_addr_masks = [0; MAX_LEVELS];
        let mut virt_addr_page_masks = [0; MAX_LEVELS];
        let mut valid_final_page_steps = [false; MAX_LEVELS];
        let mut pt_leaf_size = [0; MAX_LEVELS];
        let mut page_size_step = [0; MAX_LEVELS];
        let spare_allocs = def.spare_allocs();

        let mut i = 0;
        while i < def.virtual_address_splits.len() {
            let max = def.address_space_bits - 1;
            let min = def.virtual_address_splits[i]
                + if i == def.virtual_address_splits.len() - 1 {
                    0
                } else {
                    def.pte_size.to_le().trailing_zeros() as u8
                };
            let mask = Address::bit_mask_u8(min..max);
            pte_addr_masks[i] = mask.as_u64();

            pt_leaf_size[i] = def.pt_leaf_size(i);
            page_size_step[i] = def.page_size_step_unchecked(i);

            let (min, max) = def.virt_addr_bit_range(i);
            virt_addr_bit_ranges[i] = (min, max);
            virt_addr_masks[i] = Address::bit_mask_u8(0..(max - min - 1)).as_u64();
            virt_addr_page_masks[i] = Address::bit_mask_u8(0..(max - 1)).as_u64();

            i += 1;
        }

        i = 0;
        while i < def.valid_final_page_steps.len() {
            valid_final_page_steps[def.valid_final_page_steps[i]] = true;
            i += 1;
        }

        Self {
            def,
            pte_addr_masks,
            virt_addr_bit_ranges,
            virt_addr_masks,
            virt_addr_page_masks,
            valid_final_page_steps,
            pt_leaf_size,
            page_size_step,
            spare_allocs,
        }
    }

    pub fn pte_addr_mask(&self, pte_addr: Address, step: usize) -> u64 {
        pte_addr.as_u64() & u64::from_le(self.pte_addr_masks[step])
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
    pub(crate) fn virt_addr_filter<C, B, FO>(
        &self,
        (addr, buf): (Address, B),
        (chunks, addrs_out): (&mut TranslationChunk<C>, &mut TranslateDataVec<B>),
        fail_out: &mut FO,
    ) where
        B: SplitAtIndex,
        C: MMUTranslationBase,
        FO: Extend<(Error, Address, B)>,
    {
        vtop_trace!("total {:x}+{:x}", addr, buf.length());
        let mut tr_data = TranslateData { addr, buf };

        // Trim to virt address space limit
        let (mut left, reject) = tr_data
            .split_inclusive_at(Address::bit_mask(0..(self.def.addr_size * 8 - 1)).as_usize());

        if let Some(data) = reject {
            fail_out.extend(Some((Error::VirtualTranslate, data.addr, data.buf)));
        }

        let virt_bit_range = self.virt_addr_bit_ranges[0].1;
        let virt_range = 1u64 << (virt_bit_range - 1);
        vtop_trace!("vbr {:x} | {:x}", virt_bit_range, virt_range);
        let arch_bit_range = (!0u64) >> (64 - self.def.addr_size * 8);

        let (lower, higher) = left.split_at_address(virt_range.into());

        if let Some(mut data) = higher {
            let (reject, higher) =
                data.split_at_address_rev((arch_bit_range.wrapping_sub(virt_range)).into());

            // The upper half has to be all negative (all bits set), so compare the masks to see if
            // it is the case.
            let lhs = Address::bit_mask(virt_bit_range..(self.def.addr_size * 8 - 1)).as_u64();
            let rhs = higher.addr.as_u64() & lhs;

            if (lhs ^ rhs) == 0 {
                if higher.length() > 0 {
                    vtop_trace!("higher {:x}+{:x}", higher.addr, higher.length());
                    chunks.push_data(higher, addrs_out);
                }

                if let Some(data) = reject {
                    fail_out.extend(Some((Error::VirtualTranslate, data.addr, data.buf)));
                }
            }
        }

        if lower.length() > 0 {
            vtop_trace!("lower {:x}+{:x}", lower.addr, lower.length());
            chunks.push_data(lower, addrs_out);
        }
    }

    #[allow(unused)]
    pub fn split_count(&self) -> usize {
        self.def.virtual_address_splits.len()
    }

    pub fn pt_leaf_size(&self, step: usize) -> usize {
        self.pt_leaf_size[step]
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

    pub fn virt_addr_to_pte_offset(&self, virt_addr: Address, step: usize) -> u64 {
        u64::from_le(
            (virt_addr.as_u64().to_le() >> self.virt_addr_bit_ranges[step].0)
                & self.virt_addr_masks[step],
        ) * self.def.pte_size as u64
    }

    pub fn virt_addr_to_page_offset(&self, virt_addr: Address, step: usize) -> u64 {
        virt_addr.as_u64() & u64::from_le(self.virt_addr_page_masks[step])
    }

    /// Get the page size of a specific step without checking if such page could exist
    ///
    /// # Arguments
    ///
    /// * `step` - the current step in the page walk
    pub fn page_size_step_unchecked(&self, step: usize) -> usize {
        self.page_size_step[step]
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
        debug_assert!(self.valid_final_page_steps[step]);
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
        self.page_size_step(self.def.virtual_address_splits.len() - level)
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
                .write((self.def.writeable_bit)(pte_addr))
                .noexec((self.def.nx_bit)(pte_addr)),
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
        step == 0 || (self.def.present_bit)(pte_addr)
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
        (step == self.def.virtual_address_splits.len() - 1)
            || ((self.def.large_page_bit)(pte_addr) && self.valid_final_page_steps[step])
    }

    /// This function will do a virtual to physical memory translation for the `ArchMMUSpec` in
    /// `MMUTranslationBase` scope, over multiple elements.
    pub(crate) fn virt_to_phys_iter<T, B, D, VI, VO, FO>(
        &self,
        mem: &mut T,
        dtb: D,
        mut addrs: VI,
        out: &mut VO,
        out_fail: &mut FO,
        slice: &mut [std::mem::MaybeUninit<u8>],
    ) where
        T: PhysicalMemory + ?Sized,
        B: SplitAtIndex,
        D: MMUTranslationBase,
        VI: Iterator<Item = (Address, B)>,
        VO: Extend<(PhysicalAddress, B)>,
        FO: Extend<(Error, Address, B)>,
    {
        vtop_trace!("virt_to_phys_iter_with_mmu");

        // We need to calculate in advance how we are going to split the allocated buffer.
        // There is one important parameter `elem_count`, which determines
        // how many chunks and addresses we allow in the working stack.
        //
        // Waiting stacks are much larger, because if working stack is full from the start,
        // and it needs to be split to smaller chunks, we need space for them. In addition,
        // we need to reserve enough space for several more splits like that, because
        // the same scenario can occur for every single page mapping level.
        let chunk_size = std::mem::size_of::<TranslationChunk<TranslateData<B>>>();
        let data_size = std::mem::size_of::<TranslateData<B>>();
        let prd_size = std::mem::size_of::<PhysicalReadData>();
        let pte_size = self.def.pte_size;
        let spare_allocs = self.spare_allocs;

        let total_chunks_mul = 1 + spare_allocs;
        let working_stack_count = 2;
        let total_addr_mul = spare_allocs;

        // 2 * 8 are extra bytes for alignment in read funcs
        let elem_count = (slice.len() - 2 * 8)
            / ((total_chunks_mul + working_stack_count) * chunk_size
            + pte_size
            + prd_size
            // The +1 is for tmp_addrs
            + (total_addr_mul + working_stack_count + 1) * data_size);

        let pt1_size = 1usize << self.def.virtual_address_splits[0];

        // We need to support at least the number of entries in initial page table,
        // because one chunk can split into smaller chunks that are at least that size
        if elem_count < pt1_size {
            log::trace!(
                "input buffer may be too small! ({:x} vs {:x})",
                elem_count,
                pt1_size
            );
        }

        let waiting_chunks = elem_count * (1 + spare_allocs);
        let waiting_addr_count = elem_count * spare_allocs;

        vtop_trace!(
            "elem_count = {:x}; waiting_chunks = {:x};",
            elem_count,
            waiting_chunks
        );

        // Allocate buffers
        let (working_bytes, slice) = slice.split_at_mut(elem_count * chunk_size);
        let working_stack = MVec::from_uninit_bytes(working_bytes);
        let (working_bytes, slice) = slice.split_at_mut(elem_count * chunk_size);
        let working_stack2 = MVec::from_uninit_bytes(working_bytes);
        let (waiting_bytes, slice) = slice.split_at_mut(waiting_chunks * chunk_size);
        let waiting_stack = MVec::from_uninit_bytes(waiting_bytes);

        let (working_addrs_bytes, slice) = slice.split_at_mut(elem_count * data_size);
        let working_addrs = MVec::from_uninit_bytes(working_addrs_bytes);
        let (working_addrs_bytes, slice) = slice.split_at_mut(elem_count * data_size);
        let working_addrs2 = MVec::from_uninit_bytes(working_addrs_bytes);
        let (waiting_addrs_bytes, slice) = slice.split_at_mut(waiting_addr_count * data_size);
        let mut waiting_addrs = MVec::from_uninit_bytes(waiting_addrs_bytes);
        let (tmp_addrs_bytes, slice) = slice.split_at_mut(elem_count * data_size);
        let mut tmp_addrs = MVec::from_uninit_bytes(tmp_addrs_bytes);

        let mut working_pair = (working_stack, working_addrs);
        let mut next_working_pair = (working_stack2, working_addrs2);

        dtb.fill_init_chunk(
            &self,
            out_fail,
            &mut addrs,
            (&mut waiting_addrs, &mut tmp_addrs),
            &mut working_pair,
            &mut next_working_pair,
        );

        let mut waiting_pair = (waiting_stack, waiting_addrs);

        let buf_to_addr: fn(&[u8]) -> Address = match (self.def.endianess, self.def.pte_size) {
            (Endianess::LittleEndian, 8) => {
                |buf| Address::from(u64::from_le_bytes(buf.try_into().unwrap()))
            }
            (Endianess::LittleEndian, 4) => {
                |buf| Address::from(u32::from_le_bytes(buf.try_into().unwrap()))
            }
            (Endianess::BigEndian, 8) => {
                |buf| Address::from(u64::from_be_bytes(buf.try_into().unwrap()))
            }
            (Endianess::BigEndian, 4) => {
                |buf| Address::from(u32::from_be_bytes(buf.try_into().unwrap()))
            }
            _ => |_| Address::NULL,
        };

        while !working_pair.0.is_empty() {
            // Perform the reads here
            if let Err(err) =
                self.read_pt_address_iter(mem, &mut working_pair.0, slice, buf_to_addr)
            {
                vtop_trace!("read_pt_address_iter failure: {}", err);

                while let Some(data) = working_pair.1.pop() {
                    out_fail.extend(Some((err, data.addr, data.buf)));
                }

                while let Some(data) = waiting_pair.1.pop() {
                    out_fail.extend(Some((err, data.addr, data.buf)));
                }

                return;
            }

            self.work_through_stack(
                &mut working_pair,
                &mut next_working_pair,
                out,
                out_fail,
                &mut waiting_pair,
                &mut tmp_addrs,
            );

            debug_assert!(working_pair.1.is_empty());

            if next_working_pair.0.is_empty() {
                self.refill_stack(
                    dtb,
                    &mut working_pair,
                    &mut next_working_pair,
                    out_fail,
                    &mut addrs,
                    &mut waiting_pair,
                    &mut tmp_addrs,
                );
            } else {
                std::mem::swap(&mut working_pair, &mut next_working_pair);
            }
        }

        debug_assert!(waiting_pair.0.is_empty());
        debug_assert!(working_pair.0.is_empty());
        debug_assert!(next_working_pair.0.is_empty());
    }

    fn read_pt_address_iter<T>(
        &self,
        mem: &mut T,
        chunks: &mut TranslateVec,
        slice: &mut [std::mem::MaybeUninit<u8>],
        buf_to_addr: fn(&[u8]) -> Address,
    ) -> Result<()>
    where
        T: PhysicalMemory + ?Sized,
    {
        let pte_size = self.def.pte_size;

        let (pt_buf_bytes, slice) = slice.split_at_mut(chunks.len() * pte_size + 8);
        let mut pt_buf = MVec::from_uninit_bytes(pt_buf_bytes);
        let (pt_read_bytes, _slice) =
            slice.split_at_mut(chunks.len() * std::mem::size_of::<PhysicalReadData>() + 8);
        let mut pt_read = MVec::from_uninit_bytes(pt_read_bytes);

        pt_buf.extend((0..).map(|_| 0).take(pte_size * chunks.len()));

        for (chunk, tr_chunk) in pt_buf.chunks_exact_mut(pte_size).zip(chunks.iter()) {
            pt_read.push(PhysicalReadData(
                PhysicalAddress::with_page(
                    tr_chunk.pt_addr,
                    PageType::PAGE_TABLE,
                    self.pt_leaf_size(tr_chunk.step),
                ),
                chunk,
            ));
        }

        mem.phys_read_raw_list(&mut pt_read)?;

        for (ref mut chunk, PhysicalReadData(_, buf)) in chunks.iter_mut().zip(pt_read.iter()) {
            let pt_addr = buf_to_addr(*buf);
            chunk.pt_addr = pt_addr;
        }

        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    fn refill_stack<B: SplitAtIndex, D, VI, FO>(
        &self,
        dtb: D,
        mut working_pair: &mut (TranslateVec, TranslateDataVec<B>),
        next_working_pair: &mut (TranslateVec, TranslateDataVec<B>),
        out_fail: &mut FO,
        addrs: &mut VI,
        (waiting_stack, waiting_addrs): &mut (TranslateVec, TranslateDataVec<B>),
        tmp_addrs: &mut TranslateDataVec<B>,
    ) where
        D: MMUTranslationBase,
        FO: Extend<(Error, Address, B)>,
        VI: Iterator<Item = (Address, B)>,
    {
        let (working_stack, working_addrs) = &mut working_pair;

        // If there is a waiting stack, use it
        if !waiting_stack.is_empty() {
            while let Some(mut chunk) = waiting_stack.pop() {
                // Make sure working stack does not overflow
                if working_stack.len() >= working_stack.capacity()
                    || (working_addrs.len() + chunk.addr_count > working_stack.capacity()
                        && !working_stack.is_empty())
                {
                    waiting_stack.push(chunk);
                    break;
                } else {
                    // Move addresses between the stacks, and only until we fill up the
                    // address stack.
                    let mut new_chunk = TranslationChunk::new(chunk.pt_addr);
                    new_chunk.step = chunk.step;
                    for _ in
                        (0..chunk.addr_count).zip(working_addrs.len()..working_addrs.capacity())
                    {
                        let addr = chunk.pop_data(waiting_addrs).unwrap();
                        new_chunk.push_data(addr, working_addrs);
                    }

                    if chunk.addr_count > 0 {
                        waiting_stack.push(chunk);
                    }

                    working_stack.push(new_chunk);
                }
            }
        } else {
            dtb.fill_init_chunk(
                &self,
                out_fail,
                addrs,
                (waiting_addrs, tmp_addrs),
                working_pair,
                next_working_pair,
            );
        }
    }

    #[inline(never)]
    #[allow(clippy::too_many_arguments)]
    fn work_through_stack<B: SplitAtIndex, VO, FO>(
        &self,
        (working_stack, working_addrs): &mut (TranslateVec, TranslateDataVec<B>),
        next_working_pair: &mut (TranslateVec, TranslateDataVec<B>),
        out: &mut VO,
        out_fail: &mut FO,
        waiting_pair: &mut (TranslateVec, TranslateDataVec<B>),
        tmp_addrs: &mut TranslateDataVec<B>,
    ) where
        VO: Extend<(PhysicalAddress, B)>,
        FO: Extend<(Error, Address, B)>,
    {
        let mut prev_entry = Address::NULL;

        while let Some(mut chunk) = working_stack.pop() {
            vtop_trace!("chunk = {:#?}", &chunk);

            chunk.step += 1;

            // This is extremely important!
            // It is a something of a heuristic against
            // page tables that have all entries set to the same page table.
            //
            // For instance, windows has such global page tables, it is actually
            // just 2-3 page tables, starting from level 4 which go down one at
            // a time, covering an insane region, but not actually pointing anywhere.
            //
            // Page map chokes on these scenarios, and once again - it's page tables
            // that point nowhere! So we just try and ignore them.
            //
            // Some cases this _may_ cause issues, but it's extremely rare to have
            // 2 identical pages right next to each other. If there is ever a documented
            // case however, then we will need to workaround that.
            let pprev_entry = prev_entry;
            prev_entry = chunk.pt_addr;

            if !self.check_entry(chunk.pt_addr, chunk.step + 1) || chunk.pt_addr == pprev_entry {
                // Failure
                while let Some(entry) = chunk.pop_data(working_addrs) {
                    out_fail.extend(Some((Error::VirtualTranslate, entry.addr, entry.buf)));
                }
            } else if self.is_final_mapping(chunk.pt_addr, chunk.step) {
                // Success!
                let pt_addr = chunk.pt_addr;
                let step = chunk.step;
                while let Some(entry) = chunk.pop_data(working_addrs) {
                    out.extend(Some((
                        self.get_phys_page(pt_addr, entry.addr, step),
                        entry.buf,
                    )));
                }
            } else {
                // We still need to continue the page walk.
                // Split the chunk up into the waiting queue
                chunk.split_chunk(
                    self,
                    (working_addrs, tmp_addrs),
                    next_working_pair,
                    waiting_pair,
                );

                debug_assert!(tmp_addrs.is_empty());
            }
        }
    }
}
