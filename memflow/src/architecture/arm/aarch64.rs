use super::{
    super::{ArchitectureObj, Endianess},
    ArmArchitecture, ArmVirtualTranslate,
};

use crate::mem::virt_translate::mmu::ArchMmuDef;

use crate::types::Address;

const ARCH_4K_MMU_DEF: ArchMmuDef = ArchMmuDef {
    virtual_address_splits: &[9, 9, 9, 9, 12],
    valid_final_page_steps: &[2, 3, 4],
    address_space_bits: 48,
    endianess: Endianess::LittleEndian,
    addr_size: 8,
    pte_size: 8,
    present_bit: |a| a.bit_at(0),
    writeable_bit: |a, _| a.bit_at(10),
    nx_bit: |a, _| a.bit_at(54),
    large_page_bit: |a| !a.bit_at(1),
};

pub(super) static ARCH_SPEC: ArmArchitecture = ArmArchitecture {
    bits: 64,
    mmu: ARCH_4K_MMU_DEF.into_spec(),
};

pub static ARCH: ArchitectureObj = &ARCH_SPEC;

pub fn new_translator(dtb1: Address, dtb2: Address) -> ArmVirtualTranslate {
    ArmVirtualTranslate::new(&ARCH_SPEC, dtb1, dtb2)
}

pub(super) static ARCH_SPEC_16K: ArmArchitecture = ArmArchitecture {
    bits: 64,
    mmu: ArchMmuDef {
        virtual_address_splits: &[1, 11, 11, 11, 14],
        valid_final_page_steps: &[3, 4],
        ..ARCH_4K_MMU_DEF
    }
    .into_spec(),
};

pub static ARCH_16K: ArchitectureObj = &ARCH_SPEC_16K;

pub fn new_translator_16k(dtb1: Address, dtb2: Address) -> ArmVirtualTranslate {
    ArmVirtualTranslate::new(&ARCH_SPEC_16K, dtb1, dtb2)
}
