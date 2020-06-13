use crate::architecture::Endianess;
use crate::types::Length;

use super::ArchMMUSpec;

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
        virtual_address_splits: &[10, 10, 12],
        valid_final_page_steps: &[1, 2],
        pte_address_bits: (12, 31),
        pte_size: 4,
        present_bit: 0,
        writeable_bit: 1,
        nx_bit: 31, //Actually, NX is unsupported in x86 non-PAE, we have to do something about it
        large_page_bit: 7,
    }
}

pub fn page_size() -> Length {
    page_size_level(1)
}

pub fn page_size_level(pt_level: u32) -> Length {
    get_mmu_spec().page_size_level(pt_level as usize)
}
