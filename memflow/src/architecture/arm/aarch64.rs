use super::{
    super::{ArchMMUDef, ArchitectureObj, Endianess, ScopedVirtualTranslate},
    ARMArchitecture, ARMScopedVirtualTranslate,
};

use crate::types::Address;

pub(super) static ARCH_SPEC: ARMArchitecture = ARMArchitecture {
    bits: 64,
    mmu: ArchMMUDef {
        virtual_address_splits: &[9, 9, 9, 9, 12],
        valid_final_page_steps: &[2, 3, 4],
        address_space_bits: 52,
        endianess: Endianess::LittleEndian,
        addr_size: 8,
        pte_size: 8,
        present_bit: |a| a.bit_at(0),
        writeable_bit: |a| a.bit_at(10),
        nx_bit: |a| a.bit_at(54),
        large_page_bit: |a| !a.bit_at(1),
    }
    .into_spec(),
};

pub static ARCH: ArchitectureObj = &ARCH_SPEC;

pub fn new_translator(dtb1: Address, dtb2: Address) -> impl ScopedVirtualTranslate {
    ARMScopedVirtualTranslate::new(&ARCH_SPEC, dtb1, dtb2)
}
