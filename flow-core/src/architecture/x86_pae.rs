use super::ArchMMUSpec;
use crate::architecture::Endianess;

pub const fn bits() -> u8 {
    32
}

pub const fn endianess() -> Endianess {
    Endianess::LittleEndian
}

pub fn get_mmu_spec() -> ArchMMUSpec {
    ArchMMUSpec {
        virtual_address_splits: &[2, 9, 9, 12],
        valid_final_page_steps: &[2, 3],
        address_space_bits: 36,
        addr_size: 4,
        pte_size: 8,
        present_bit: 0,
        writeable_bit: 1,
        nx_bit: 63,
        large_page_bit: 7,
    }
}

pub fn page_size() -> usize {
    page_size_level(1)
}

pub fn page_size_level(pt_level: u32) -> usize {
    get_mmu_spec().page_size_level(pt_level as usize)
}

//x64 tests MMU rigorously, here we will only test a few special cases
#[cfg(test)]
mod tests {
    use super::super::mmu_spec::masks::*;
    use super::get_mmu_spec;
    use crate::types::{size, Address};

    #[test]
    fn x86_pae_pte_bitmasks() {
        let mmu = get_mmu_spec();
        let mask_addr = Address::invalid();
        assert_eq!(mmu.pte_addr_mask(mask_addr, 0), make_bit_mask(5, 35));
        assert_eq!(mmu.pte_addr_mask(mask_addr, 1), make_bit_mask(12, 35));
        assert_eq!(mmu.pte_addr_mask(mask_addr, 2), make_bit_mask(12, 35));
    }

    #[test]
    fn x86_pae_pte_leaf_size() {
        let mmu = get_mmu_spec();
        assert_eq!(mmu.pt_leaf_size(0), 32);
        assert_eq!(mmu.pt_leaf_size(1), size::kb(4));
    }

    #[test]
    fn x86_pae_page_size_level() {
        let mmu = get_mmu_spec();
        assert_eq!(mmu.page_size_level(1), size::kb(4));
        assert_eq!(mmu.page_size_level(2), size::mb(2));
    }
}
