use super::{
    super::{ArchMMUSpec, ArchitectureObj, Endianess, ScopedVirtualTranslate},
    X86Architecture, X86ScopedVirtualTranslate,
};

use crate::types::Address;

pub(super) const ARCH_SPEC: X86Architecture = X86Architecture {
    bits: 32,
    endianess: Endianess::LittleEndian,
    mmu: ArchMMUSpec {
        virtual_address_splits: &[2, 9, 9, 12],
        valid_final_page_steps: &[2, 3],
        address_space_bits: 36,
        addr_size: 4,
        pte_size: 8,
        present_bit: 0,
        writeable_bit: 1,
        nx_bit: 63,
        large_page_bit: 7,
    },
};

pub static ARCH: ArchitectureObj = &ARCH_SPEC;

pub fn new_translator(dtb: Address) -> impl ScopedVirtualTranslate {
    X86ScopedVirtualTranslate::new(&ARCH_SPEC, dtb)
}

//x64 tests MMU rigorously, here we will only test a few special cases
#[cfg(test)]
mod tests {
    use crate::architecture::mmu_spec::ArchMMUSpec;
    use crate::types::{size, Address};

    fn get_mmu_spec() -> ArchMMUSpec {
        super::ARCH_SPEC.mmu
    }

    #[test]
    fn x86_pae_pte_bitmasks() {
        let mmu = get_mmu_spec();
        let mask_addr = Address::invalid();
        assert_eq!(
            mmu.pte_addr_mask(mask_addr, 0),
            Address::bit_mask(5..35).as_u64()
        );
        assert_eq!(
            mmu.pte_addr_mask(mask_addr, 1),
            Address::bit_mask(12..35).as_u64()
        );
        assert_eq!(
            mmu.pte_addr_mask(mask_addr, 2),
            Address::bit_mask(12..35).as_u64()
        );
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
