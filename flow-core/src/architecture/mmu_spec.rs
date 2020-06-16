#[macro_use]
mod masks;
use masks::*;

use log::trace;

use crate::types::{Address, Length, PageType, PhysicalAddress};

pub struct ArchMMUSpec {
    pub virtual_address_splits: &'static [u8],
    pub valid_final_page_steps: &'static [usize],
    pub address_space_bits: u8,
    pub pte_size: usize,
    pub present_bit: u8,
    pub writeable_bit: u8,
    pub nx_bit: u8,
    pub large_page_bit: u8,
}

impl ArchMMUSpec {
    pub const fn new(
        virtual_address_splits: &'static [u8],
        valid_final_page_steps: &'static [usize],
        address_space_bits: u8,
        pte_size: usize,
        present_bit: u8,
        writeable_bit: u8,
        nx_bit: u8,
        large_page_bit: u8,
    ) -> ArchMMUSpec {
        ArchMMUSpec {
            virtual_address_splits,
            valid_final_page_steps,
            address_space_bits,
            pte_size,
            present_bit,
            writeable_bit,
            nx_bit,
            large_page_bit,
        }
    }

    pub fn pte_addr_mask(&self, pte_addr: Address, step: usize) -> u64 {
        let max = self.address_space_bits - 1;
        let min = self.virtual_address_splits[step]
            + if step == self.virtual_address_splits.len() - 1 {
                0
            } else {
                self.pte_size.to_le().trailing_zeros() as u8
            };
        let mask = make_bit_mask(min, max);
        if cfg!(feature = "trace_mmu") {
            trace!("pte_addr_mask={:b}", mask);
        }
        pte_addr.as_u64() & mask
    }

    fn virt_addr_bit_range(&self, step: usize) -> (u8, u8) {
        let max_index_bits = self.virtual_address_splits[step..].iter().sum::<u8>();
        let min_index_bits = max_index_bits - self.virtual_address_splits[step];
        (min_index_bits, max_index_bits)
    }

    fn virt_addr_to_pte_offset(&self, virt_addr: Address, step: usize) -> u64 {
        let (min, max) = self.virt_addr_bit_range(step);
        if cfg!(feature = "trace_mmu") {
            trace!("virt_addr_bit_range for step {} = ({}, {})", step, min, max);
        }

        let shifted = virt_addr.as_u64() >> min;
        let mask = make_bit_mask(0, max - min - 1);

        (shifted & mask) * self.pte_size as u64
    }

    fn virt_addr_to_page_offset(&self, virt_addr: Address, step: usize) -> u64 {
        let max = self.virt_addr_bit_range(step).1;
        virt_addr.as_u64() & make_bit_mask(0, max - 1)
    }

    pub fn split_count(&self) -> usize {
        self.virtual_address_splits.len()
    }

    pub fn pt_leaf_size(&self, step: usize) -> Length {
        let (min, max) = self.virt_addr_bit_range(step);
        Length::from((1 << (max - min)) * self.pte_size)
    }

    pub fn vtop_step(&self, pte_addr: Address, virt_addr: Address, step: usize) -> Address {
        Address::from(
            self.pte_addr_mask(pte_addr, step) | self.virt_addr_to_pte_offset(virt_addr, step),
        )
    }

    pub fn page_size_step_unchecked(&self, step: usize) -> Length {
        let max_index_bits = self.virtual_address_splits[step..].iter().sum::<u8>();
        Length::from(1u64 << max_index_bits)
    }

    pub fn page_size_step(&self, step: usize) -> Length {
        debug_assert!(self.valid_final_page_steps.binary_search(&step).is_ok());
        self.page_size_step_unchecked(step)
    }

    pub fn page_size_level(&self, level: usize) -> Length {
        self.page_size_step(self.virtual_address_splits.len() - level)
    }

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
            PageType::from_writeable_bit(get_bit!(pte_addr.as_u64(), self.writeable_bit)),
            self.page_size_step(step),
        )
    }

    pub fn check_entry(&self, pte_addr: Address, step: usize) -> bool {
        step == 0 || get_bit!(pte_addr.as_u64(), self.present_bit)
    }

    pub fn is_final_mapping(&self, pte_addr: Address, step: usize) -> bool {
        (step == self.virtual_address_splits.len() - 1)
            || (get_bit!(pte_addr.as_u64(), self.large_page_bit)
                && self.valid_final_page_steps.binary_search(&step).is_ok())
    }
}
