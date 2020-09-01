use super::{
    super::{ArchMMUSpec, ArchitectureObj, Endianess, ScopedVirtualTranslate},
    X86Architecture, X86ScopedVirtualTranslate,
};

use crate::types::Address;

pub(super) const ARCH_SPEC: X86Architecture = X86Architecture {
    bits: 64,
    endianess: Endianess::LittleEndian,
    mmu: ArchMMUSpec {
        virtual_address_splits: &[9, 9, 9, 9, 12],
        valid_final_page_steps: &[2, 3, 4],
        address_space_bits: 52,
        addr_size: 8,
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

#[cfg(test)]
mod tests {
    use crate::architecture::mmu_spec::ArchMMUSpec;
    use crate::types::{size, Address, PageType};

    fn get_mmu_spec() -> ArchMMUSpec {
        super::ARCH_SPEC.mmu
    }

    #[test]
    fn x64_pte_bitmasks() {
        let mmu = get_mmu_spec();
        let mask_addr = Address::invalid();
        assert_eq!(
            mmu.pte_addr_mask(mask_addr, 0),
            Address::bit_mask(12..51).as_u64()
        );
        assert_eq!(
            mmu.pte_addr_mask(mask_addr, 1),
            Address::bit_mask(12..51).as_u64()
        );
        assert_eq!(
            mmu.pte_addr_mask(mask_addr, 2),
            Address::bit_mask(12..51).as_u64()
        );
        assert_eq!(
            mmu.pte_addr_mask(mask_addr, 3),
            Address::bit_mask(12..51).as_u64()
        );
    }

    #[test]
    fn x64_split_count() {
        assert_eq!(get_mmu_spec().split_count(), 5);
    }

    #[test]
    fn x64_pte_leaf_size() {
        let mmu = get_mmu_spec();
        assert_eq!(mmu.pt_leaf_size(0), size::kb(4));
    }

    #[test]
    fn x64_page_size_level() {
        let mmu = get_mmu_spec();
        assert_eq!(mmu.page_size_level(1), size::kb(4));
        assert_eq!(mmu.page_size_level(2), size::mb(2));
        assert_eq!(mmu.page_size_level(3), size::gb(1));
    }

    #[test]
    fn x64_page_size_step() {
        let mmu = get_mmu_spec();
        assert_eq!(mmu.page_size_step(2), size::gb(1));
        assert_eq!(mmu.page_size_step(3), size::mb(2));
        assert_eq!(mmu.page_size_step(4), size::kb(4));
    }

    #[test]
    #[should_panic]
    #[cfg(debug_assertions)]
    fn x64_page_size_level_4() {
        let mmu = get_mmu_spec();
        assert_eq!(mmu.page_size_level(4), size::gb(512));
    }

    #[test]
    #[should_panic]
    #[cfg(debug_assertions)]
    fn x64_page_size_level_5() {
        let mmu = get_mmu_spec();
        assert_eq!(mmu.page_size_level(5), size::gb(512 * 512));
    }

    #[test]
    fn x64_vtop_step() {
        let mmu = get_mmu_spec();
        let indices = [145_usize, 54, 64, 0];
        let virt_address = indices
            .iter()
            .rev()
            .map(|i| *i as u64)
            .enumerate()
            .fold(0, |state, (lvl, idx)| state | (idx << (12 + 9 * lvl)))
            .into();
        let pte_address = Address::from(size::kb(4 * 45));
        assert_eq!(
            mmu.vtop_step(pte_address, virt_address, 0),
            pte_address + (indices[0] * 8)
        );
        assert_eq!(
            mmu.vtop_step(pte_address, virt_address, 1),
            pte_address + (indices[1] * 8)
        );
        assert_eq!(
            mmu.vtop_step(pte_address, virt_address, 2),
            pte_address + (indices[2] * 8)
        );
        assert_eq!(
            mmu.vtop_step(pte_address, virt_address, 3),
            pte_address + (indices[3] * 8)
        );
    }

    #[test]
    fn x64_get_phys_page() {
        let mmu = get_mmu_spec();
        let indices = [145_usize, 54, 64, 21];
        let page_offset = 1243_usize;
        let virt_address = indices
            .iter()
            .rev()
            .map(|i| *i as u64)
            .enumerate()
            .fold(page_offset as u64, |state, (lvl, idx)| {
                state | (idx << (12 + 9 * lvl))
            })
            .into();
        let pte_address = Address::from(size::gb(57));

        assert_eq!(
            mmu.get_phys_page(pte_address, virt_address, 4).page_type(),
            PageType::READ_ONLY
        );
        assert_eq!(
            mmu.get_phys_page(pte_address, virt_address, 4).page_size(),
            size::kb(4)
        );
        assert_eq!(
            mmu.get_phys_page(pte_address, virt_address, 2).page_base(),
            pte_address
        );

        assert_eq!(
            mmu.get_phys_page(pte_address, virt_address, 4).address(),
            pte_address + page_offset
        );
        assert_eq!(
            mmu.get_phys_page(pte_address, virt_address, 3).address(),
            pte_address + size::kb(4 * indices[3]) + page_offset
        );
        assert_eq!(
            mmu.get_phys_page(pte_address, virt_address, 2).address(),
            pte_address + size::mb(2 * indices[2]) + size::kb(4 * indices[3]) + page_offset
        );
    }

    #[test]
    fn x64_check_entry() {
        let mmu = get_mmu_spec();
        let pte_address = 1.into();
        assert_eq!(mmu.check_entry(pte_address, 0), true);
        assert_eq!(mmu.check_entry(pte_address, 1), true);
        assert_eq!(mmu.check_entry(pte_address, 2), true);
        assert_eq!(mmu.check_entry(pte_address, 3), true);
        assert_eq!(mmu.check_entry(pte_address, 4), true);
        let pte_address = 0.into();
        assert_eq!(mmu.check_entry(pte_address, 0), true);
        assert_eq!(mmu.check_entry(pte_address, 3), false);
    }

    #[test]
    fn x64_is_final_mapping() {
        let mmu = get_mmu_spec();
        let pte_address = (1 << 7).into();
        assert_eq!(mmu.is_final_mapping(pte_address, 0), false);
        assert_eq!(mmu.is_final_mapping(pte_address, 1), false);
        assert_eq!(mmu.is_final_mapping(pte_address, 2), true);
        assert_eq!(mmu.is_final_mapping(pte_address, 3), true);
        assert_eq!(mmu.is_final_mapping(pte_address, 4), true);
        let pte_address = 0.into();
        assert_eq!(mmu.is_final_mapping(pte_address, 0), false);
        assert_eq!(mmu.is_final_mapping(pte_address, 1), false);
        assert_eq!(mmu.is_final_mapping(pte_address, 2), false);
        assert_eq!(mmu.is_final_mapping(pte_address, 3), false);
        assert_eq!(mmu.is_final_mapping(pte_address, 4), true);
    }
}
