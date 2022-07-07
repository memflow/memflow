use super::{
    super::{ArchitectureObj, Endianess},
    X86Architecture, X86VirtualTranslate,
};

use crate::mem::virt_translate::mmu::ArchMmuDef;

use crate::types::Address;

pub(super) static ARCH_SPEC: X86Architecture = X86Architecture {
    bits: 32,
    mmu: ArchMmuDef {
        virtual_address_splits: &[2, 9, 9, 12],
        valid_final_page_steps: &[2, 3],
        address_space_bits: 36,
        endianess: Endianess::LittleEndian,
        addr_size: 4,
        pte_size: 8,
        present_bit: |a| a.bit_at(0),
        writeable_bit: |a, pb| pb || a.bit_at(1),
        nx_bit: |a, pb| pb || a.bit_at(63),
        large_page_bit: |a| a.bit_at(7),
    }
    .into_spec(),
};

pub static ARCH: ArchitectureObj = &ARCH_SPEC;

pub fn new_translator(dtb: Address) -> X86VirtualTranslate {
    X86VirtualTranslate::new(&ARCH_SPEC, dtb)
}

//x64 tests MMU rigorously, here we will only test a few special cases
#[cfg(test)]
mod tests {
    use crate::mem::virt_translate::mmu::ArchMmuSpec;
    use crate::types::{mem, size, Address};

    fn get_mmu_spec() -> &'static ArchMmuSpec {
        &super::ARCH_SPEC.mmu
    }

    #[test]
    fn x86_pae_pte_bitmasks() {
        let mmu = get_mmu_spec();
        let mask_addr = Address::invalid();
        assert_eq!(
            mmu.pte_addr_mask(mask_addr, 0),
            Address::bit_mask(5..=35).to_umem()
        );
        assert_eq!(
            mmu.pte_addr_mask(mask_addr, 1),
            Address::bit_mask(12..=35).to_umem()
        );
        assert_eq!(
            mmu.pte_addr_mask(mask_addr, 2),
            Address::bit_mask(12..=35).to_umem()
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
        assert_eq!(mmu.page_size_level(1), mem::kb(4));
        assert_eq!(mmu.page_size_level(2), mem::mb(2));
    }
}
