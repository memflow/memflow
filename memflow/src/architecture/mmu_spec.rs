pub(crate) mod translate_data;

use crate::error::{Error, Result};
use crate::iter::SplitAtIndex;
use crate::mem::{PhysicalMemory, PhysicalReadData};
use crate::types::{Address, PageType, PhysicalAddress};
use std::convert::TryInto;
use translate_data::{TranslateData, TranslateDataVec, TranslateVec, TranslationChunk};

use bumpalo::Bump;

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

pub trait MMUTranslationBase: Clone + Copy + core::fmt::Debug {
    // Retrieves page table address by virtual address
    fn get_pt_by_virt_addr(&self, address: Address) -> Address;

    // Retreives page table address, and its index by index within
    // For instance, on ARM index 257 would return kernel page table
    // address, and index 1. On X86, however, this is a no-op that returns
    // underlying page table Address and `idx`.
    fn get_pt_by_index(&self, idx: usize) -> (Address, usize);

    fn pt_count(&self) -> usize;

    fn virt_addr_filter<B: SplitAtIndex, O: Extend<(Error, Address, B)>>(
        &self,
        spec: &ArchMMUSpec,
        addr: (Address, B),
        work_group: (&mut TranslationChunk<Self>, &mut TranslateDataVec<B>),
        out_fail: &mut O,
    );

    fn fill_init_chunk<VI, FO, B>(
        &self,
        spec: &ArchMMUSpec,
        out_fail: &mut FO,
        addrs: &mut VI,
        (waiting_addrs, tmp_addrs): (&mut TranslateDataVec<B>, &mut TranslateDataVec<B>),
        work_vecs: (&mut TranslateVec, &mut TranslateDataVec<B>),
        working_addr_count: usize,
    ) where
        VI: Iterator<Item = (Address, B)>,
        FO: Extend<(Error, Address, B)>,
        B: SplitAtIndex,
    {
        let mut init_chunk = TranslationChunk::new(*self);

        for (_, data) in (0..working_addr_count).zip(addrs) {
            self.virt_addr_filter(spec, data, (&mut init_chunk, waiting_addrs), out_fail);
        }

        if init_chunk.addr_count > 0 {
            vtop_trace!("init_chunk = {:#?}", &init_chunk);
            init_chunk.split_chunk(spec, (waiting_addrs, tmp_addrs), work_vecs);
        }
    }
}

impl MMUTranslationBase for Address {
    fn get_pt_by_virt_addr(&self, _: Address) -> Address {
        *self
    }

    fn get_pt_by_index(&self, idx: usize) -> (Address, usize) {
        (*self, idx)
    }

    fn pt_count(&self) -> usize {
        1
    }

    fn virt_addr_filter<B, O>(
        &self,
        spec: &ArchMMUSpec,
        addr: (Address, B),
        work_group: (&mut TranslationChunk<Self>, &mut TranslateDataVec<B>),
        out_fail: &mut O,
    ) where
        B: SplitAtIndex,
        O: Extend<(Error, Address, B)>,
    {
        spec.virt_addr_filter(addr, work_group, out_fail);
    }

    //TODO: Optimized fill_init_vec impl for non-split page tables
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
        let (mut left, reject) =
            tr_data.split_inclusive_at(Address::bit_mask(0..(self.addr_size * 8 - 1)).as_usize());

        if let Some(data) = reject {
            fail_out.extend(Some((Error::VirtualTranslate, data.addr, data.buf)));
        }

        let virt_bit_range = self.virt_addr_bit_range(0).1;
        let virt_range = 1u64 << (virt_bit_range - 1);
        vtop_trace!("vbr {:x} | {:x}", virt_bit_range, virt_range);

        let (lower, higher) = left.split_at_address(virt_range.into());

        if let Some(mut data) = higher {
            let (reject, higher) =
                data.split_at_address_rev((0u64.wrapping_sub(virt_range).wrapping_sub(1)).into());

            // The upper half has to be all negative (all bits set), so compare the masks to see if
            // it is the case.
            let lhs = Address::bit_mask(virt_bit_range..(self.addr_size * 8 - 1)).as_u64();
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

    pub fn spare_allocs(&self) -> usize {
        let mut iter = self.virtual_address_splits.iter();
        let _ = iter.next_back();
        iter.fold(0, |acc, x| acc + (1 << x))
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
        mut addrs: VI,
        out: &mut VO,
        out_fail: &mut FO,
        _arena: &Bump,
    ) where
        T: PhysicalMemory + ?Sized,
        B: SplitAtIndex,
        D: MMUTranslationBase,
        VI: Iterator<Item = (Address, B)>,
        VO: Extend<(PhysicalAddress, B)>,
        FO: Extend<(Error, Address, B)>,
    {
        vtop_trace!("virt_to_phys_iter_with_mmu");

        let spare_chunks = 0x100;
        let total_chunks = spare_chunks + spare_chunks * self.spare_allocs();

        //log::debug!("SC {:x} SA {:x} | {:x} {:x} {:x}", spare_chunks, self.spare_allocs(), total_chunks, spare_chunks * self.spare_allocs(), spare_chunks + spare_chunks * self.spare_allocs());

        let working_addr_count = 0x100;

        vtop_trace!(
            "spare_chunks = {:x}; total_chunks = {:x}; working_addr_count = {:x}",
            spare_chunks,
            total_chunks,
            working_addr_count
        );

        // Chunk has the step, pt address, and address count within.
        // Addresses are in chunks of 64, to both be optimal and versatile

        // Pop stuff in directly to working stack,
        // use waiting stack only later
        let mut working_stack = Vec::with_capacity(spare_chunks);
        let mut waiting_stack = Vec::with_capacity(total_chunks);

        let mut pt_buf = Vec::with_capacity(spare_chunks);
        let mut pt_read = Vec::with_capacity(spare_chunks);

        let mut working_addrs = Vec::with_capacity(working_addr_count);
        let mut waiting_addrs = vec![];
        let mut tmp_addrs = vec![];

        let mut mc_work_chunks = 0;
        let mut mc_wait_chunks = 0;
        let mut mc_work_addrs = 0;
        let mut mc_wait_addrs = 0;

        dtb.fill_init_chunk(
            &self,
            out_fail,
            &mut addrs,
            (&mut waiting_addrs, &mut tmp_addrs),
            (&mut working_stack, &mut working_addrs),
            working_addr_count,
        );

        //log::trace!("{:x} {:x} {:x} {:x}", waiting_addrs.len(), waiting_stack.len(), working_addrs.len(), working_stack.len());

        while !working_stack.is_empty() {
            // Perform the reads here
            if let Err(err) =
                self.read_pt_address_iter(mem, &mut working_stack, &mut pt_buf, &mut pt_read)
            {
                vtop_trace!("read_pt_address_iter failure: {}", err);
                out_fail.extend(
                    working_addrs
                        .into_iter()
                        .chain(waiting_addrs.into_iter())
                        .map(|data| (err, data.addr, data.buf)),
                );
                return;
            }

            let mut prev_entry = Address::NULL;

            mc_work_addrs = std::cmp::max(mc_work_addrs, working_addrs.len());
            mc_work_chunks = std::cmp::max(mc_work_chunks, working_stack.len());

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

                if !self.check_entry(chunk.pt_addr, chunk.step + 1) || chunk.pt_addr == pprev_entry
                {
                    out_fail.extend(
                        chunk
                            .into_addr_iter(&mut working_addrs)
                            .map(|entry| (Error::VirtualTranslate, entry.addr, entry.buf)),
                    );
                } else if self.is_final_mapping(chunk.pt_addr, chunk.step) {
                    let pt_addr = chunk.pt_addr;
                    let step = chunk.step;
                    out.extend(
                        chunk.into_addr_iter(&mut working_addrs).map(|entry| {
                            (self.get_phys_page(pt_addr, entry.addr, step), entry.buf)
                        }),
                    );
                } else {
                    // We still need to continue the page walk.
                    // Split the chunk up into the waiting queue
                    chunk.split_chunk(
                        self,
                        (&mut working_addrs, &mut tmp_addrs),
                        (&mut waiting_stack, &mut waiting_addrs),
                    );

                    debug_assert!(tmp_addrs.is_empty());

                    /*{
                        let mut addr_iter = working_addrs.iter().rev();
                        for (i, c) in working_stack.iter().rev().enumerate() {
                            assert!(c.verify_bounds(&mut addr_iter), "PPWO {} {:#?}", i, c);
                        }
                    }

                    {
                        let mut addr_iter = waiting_addrs.iter().rev();
                        for (i, c) in waiting_stack.iter().rev().enumerate() {
                            assert!(c.verify_bounds(&mut addr_iter), "PPWA {} {:#?}", i, c);
                        }
                    }*/
                }
            }

            debug_assert!(working_addrs.is_empty());

            mc_wait_addrs = std::cmp::max(mc_wait_addrs, waiting_addrs.len());
            mc_wait_chunks = std::cmp::max(mc_wait_chunks, waiting_stack.len());

            if !waiting_stack.is_empty() {
                while let Some(mut chunk) = waiting_stack.pop() {
                    if working_stack.len() >= spare_chunks
                        || (working_addrs.len() + chunk.addr_count > working_addr_count
                            && !working_stack.is_empty())
                    {
                        waiting_stack.push(chunk);
                        break;
                    } else {
                        let mut new_chunk = TranslationChunk::new(chunk.pt_addr);
                        new_chunk.step = chunk.step;
                        for _ in (0..chunk.addr_count).zip(working_addrs.len()..working_addr_count)
                        {
                            let addr = chunk.pop_data(&mut waiting_addrs).unwrap();
                            new_chunk.push_data(addr, &mut working_addrs);
                        }

                        if chunk.addr_count > 0 {
                            waiting_stack.push(chunk);
                        }

                        working_stack.push(new_chunk);
                    }
                }
            } else {
                // TODO: Maybe feed it in directly?? Idk. Probably want to do it if <50% addresses
                // are full after waiting stack is done
                dtb.fill_init_chunk(
                    &self,
                    out_fail,
                    &mut addrs,
                    (&mut waiting_addrs, &mut tmp_addrs),
                    (&mut working_stack, &mut working_addrs),
                    working_addr_count,
                );
            }
        }

        debug_assert!(waiting_stack.is_empty());
    }

    fn read_pt_address_iter<T>(
        &self,
        mem: &mut T,
        chunks: &mut TranslateVec,
        pt_buf: &mut Vec<u8>,
        pt_read: &mut Vec<PhysicalReadData>,
    ) -> Result<()>
    where
        T: PhysicalMemory + ?Sized,
    {
        //TODO: use self.pt_leaf_size(step) (need to handle LittleEndian::read_u64)
        let pte_size = 8;

        //pt_buf.clear();
        pt_buf.resize(pte_size * chunks.len(), 0);

        debug_assert!(pt_read.is_empty());

        //This is safe, because pt_read gets cleared at the end of the function
        let pt_read: &mut Vec<PhysicalReadData> = unsafe { std::mem::transmute(pt_read) };

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

        mem.phys_read_raw_list(pt_read)?;

        for (ref mut chunk, PhysicalReadData(_, buf)) in chunks.iter_mut().zip(pt_read.iter()) {
            let pt_addr = Address::from(u64::from_le_bytes(buf[0..8].try_into().unwrap()));
            chunk.pt_addr = pt_addr;
        }

        pt_read.clear();

        Ok(())
    }
}
