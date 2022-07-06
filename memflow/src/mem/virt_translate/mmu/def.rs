use super::ArchMmuSpec;
use crate::architecture::Endianess;
use crate::types::{clamp_to_usize, umem, Address};

/// The `ArchMmuDef` structure defines how a real memory management unit should behave when
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
pub struct ArchMmuDef {
    /// defines the way virtual addresses gets split (the last element
    /// being the final physical page offset, and thus treated a bit differently)
    pub virtual_address_splits: &'static [u8],
    /// defines at which page mapping steps we can return a large page.
    /// Steps are indexed from 0, and the list has to be sorted, otherwise the code may fail.
    pub valid_final_page_steps: &'static [usize],
    /// define the address space upper bound (32 for x86, 52 for x86_64)
    pub address_space_bits: u8,
    /// Defines the byte order of the architecture
    pub endianess: Endianess,
    /// native pointer size in bytes for the architecture.
    pub addr_size: u8,
    /// size of an individual page table entry in bytes.
    pub pte_size: usize,
    /// index of a bit in PTE defining whether the page is present or not.
    pub present_bit: fn(Address) -> bool,
    /// index of a bit in PTE defining if the page is writeable.
    pub writeable_bit: fn(Address, bool) -> bool,
    /// index of a bit in PTE defining if the page is non-executable.
    pub nx_bit: fn(Address, bool) -> bool,
    /// function for checking a bit in PTE to see if the PTE points to a large page.
    pub large_page_bit: fn(Address) -> bool,
}

impl ArchMmuDef {
    pub const fn into_spec(self) -> ArchMmuSpec {
        ArchMmuSpec::from_def(self)
    }

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
    #[allow(unused)]
    pub fn pte_addr_mask(&self, pte_addr: Address, step: usize) -> umem {
        let max = self.address_space_bits - 1;
        let min = self.virtual_address_splits[step]
            + if step == self.virtual_address_splits.len() - 1 {
                0
            } else {
                self.pte_size.to_le().trailing_zeros() as u8
            };
        let mask = Address::bit_mask(min..=max);
        pte_addr.to_umem() & umem::from_le(mask.to_umem())
    }

    pub(crate) const fn virt_addr_bit_range(&self, step: usize) -> (u8, u8) {
        let max_index_bits = {
            let subsl = &self.virtual_address_splits;
            let mut accum = 0;
            let mut i = step;
            while i < subsl.len() {
                accum += subsl[i];
                i += 1;
            }
            accum
        };
        let min_index_bits = max_index_bits - self.virtual_address_splits[step];
        (min_index_bits, max_index_bits)
    }

    /// Return the number of splits of virtual addresses
    ///
    /// The returned value will be one more than the number of page table levels
    #[allow(unused)]
    pub fn split_count(&self) -> usize {
        self.virtual_address_splits.len()
    }

    /// Returns the upper bound of number of splits that can occur when performing translation
    pub const fn spare_allocs(&self) -> usize {
        let mut i = 1;
        let mut fold = 0;
        while i < self.virtual_address_splits.len() {
            fold += 1 << self.virtual_address_splits[i - 1];
            i += 1;
        }
        fold
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
    pub const fn pt_leaf_size(&self, step: usize) -> usize {
        let (min, max) = self.virt_addr_bit_range(step);
        clamp_to_usize((1 << (max - min)) * self.pte_size as umem)
    }

    /// Get the page size of a specific step without checking if such page could exist
    ///
    /// # Arguments
    ///
    /// * `step` - the current step in the page walk
    #[allow(unused)]
    pub const fn page_size_step_unchecked(&self, step: usize) -> umem {
        let max_index_bits = {
            let subsl = &self.virtual_address_splits;
            let mut i = step;
            let mut accum = 0;
            while i < subsl.len() {
                accum += subsl[i];
                i += 1;
            }
            accum
        };
        1 << max_index_bits
    }

    /// Get the page size of a specific page walk step
    ///
    /// This function is preferable to use externally, because in debug builds it will check if such
    /// page could exist, and if can not, it will panic
    ///
    /// # Arguments
    ///
    /// * `step` - the current step in the page walk
    #[allow(unused)]
    pub fn page_size_step(&self, step: usize) -> umem {
        debug_assert!(self.valid_final_page_steps.binary_search(&step).is_ok());
        self.page_size_step_unchecked(step)
    }
}
