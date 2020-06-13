use super::ArchMMUSpec;
use crate::architecture::Endianess;
use crate::types::Length;

pub fn bits() -> u8 {
    32
}

pub fn endianess() -> Endianess {
    Endianess::LittleEndian
}

pub fn len_addr() -> Length {
    Length::from(4)
}

pub fn get_mmu_spec() -> ArchMMUSpec {
    ArchMMUSpec {
        virtual_address_splits: &[2, 9, 9, 12],
        valid_final_page_steps: &[2, 3],
        pte_address_bits: (12, 35),
        pte_size: 8,
        present_bit: 0,
        writeable_bit: 1,
        nx_bit: 63,
        large_page_bit: 7,
    }
}

pub fn page_size() -> Length {
    page_size_level(1)
}

pub fn page_size_level(pt_level: u32) -> Length {
    get_mmu_spec().page_size_level(pt_level as usize)
}
