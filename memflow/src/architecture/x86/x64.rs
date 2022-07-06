use super::{
    super::{ArchitectureObj, Endianess},
    X86Architecture, X86VirtualTranslate,
};

use crate::mem::virt_translate::mmu::ArchMmuDef;

use crate::types::Address;

pub(super) static ARCH_SPEC: X86Architecture = X86Architecture {
    bits: 64,
    mmu: ArchMmuDef {
        virtual_address_splits: &[9, 9, 9, 9, 12],
        valid_final_page_steps: &[2, 3, 4],
        address_space_bits: 52,
        endianess: Endianess::LittleEndian,
        addr_size: 8,
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

#[cfg(test)]
mod tests {
    use crate::mem::virt_translate::mmu::{ArchMmuSpec, FlagsType};
    use crate::types::{mem, size, umem, Address, PageType};

    fn get_mmu_spec() -> &'static ArchMmuSpec {
        &super::ARCH_SPEC.mmu
    }

    #[test]
    fn x64_pte_bitmasks() {
        let mmu = get_mmu_spec();
        let mask_addr = Address::invalid();
        assert_eq!(
            mmu.pte_addr_mask(mask_addr, 0),
            Address::bit_mask(12..=51).to_umem()
        );
        assert_eq!(
            mmu.pte_addr_mask(mask_addr, 1),
            Address::bit_mask(12..=51).to_umem()
        );
        assert_eq!(
            mmu.pte_addr_mask(mask_addr, 2),
            Address::bit_mask(12..=51).to_umem()
        );
        assert_eq!(
            mmu.pte_addr_mask(mask_addr, 3),
            Address::bit_mask(12..=51).to_umem()
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
        assert_eq!(mmu.page_size_level(1), mem::kb(4));
        assert_eq!(mmu.page_size_level(2), mem::mb(2));
        assert_eq!(mmu.page_size_level(3), mem::gb(1));
    }

    #[test]
    fn x64_page_size_step() {
        let mmu = get_mmu_spec();
        assert_eq!(mmu.page_size_step(2), mem::gb(1));
        assert_eq!(mmu.page_size_step(3), mem::mb(2));
        assert_eq!(mmu.page_size_step(4), mem::kb(4));
    }

    #[test]
    #[should_panic]
    #[cfg(debug_assertions)]
    fn x64_page_size_level_4() {
        let mmu = get_mmu_spec();
        assert_eq!(mmu.page_size_level(4), mem::gb(512));
    }

    #[test]
    #[should_panic]
    #[cfg(debug_assertions)]
    fn x64_page_size_level_5() {
        let mmu = get_mmu_spec();
        assert_eq!(mmu.page_size_level(5), mem::gb(512 * 512));
    }

    #[test]
    fn x64_vtop_step() {
        let mmu = get_mmu_spec();
        let indices = [145_usize, 54, 64, 0];
        let virt_address = indices
            .iter()
            .rev()
            .map(|i| *i as umem)
            .enumerate()
            .fold(0, |state, (lvl, idx)| state | (idx << (12 + 9 * lvl)))
            .into();
        let pte_address = Address::from(mem::kb(4 * 45));
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
        let indices: [umem; 4] = [145, 54, 64, 21];
        let page_offset: umem = 1243;
        let virt_address = indices
            .iter()
            .rev()
            .map(|i| *i as umem)
            .enumerate()
            .fold(page_offset as umem, |state, (lvl, idx)| {
                state | (idx << (12 + 9 * lvl))
            })
            .into();
        let pte_address = Address::from(mem::gb(57));
        let prev_flags = FlagsType::NONE;

        assert_eq!(
            mmu.get_phys_page(pte_address, virt_address, 4, prev_flags)
                .page_type(),
            PageType::READ_ONLY
        );
        assert_eq!(
            mmu.get_phys_page(pte_address, virt_address, 4, prev_flags)
                .page_size(),
            mem::kb(4)
        );
        assert_eq!(
            mmu.get_phys_page(pte_address, virt_address, 2, prev_flags)
                .page_base(),
            pte_address
        );

        assert_eq!(
            mmu.get_phys_page(pte_address, virt_address, 4, prev_flags)
                .address(),
            pte_address + page_offset
        );
        assert_eq!(
            mmu.get_phys_page(pte_address, virt_address, 3, prev_flags)
                .address(),
            pte_address + mem::kb(4 * indices[3]) + page_offset
        );
        assert_eq!(
            mmu.get_phys_page(pte_address, virt_address, 2, prev_flags)
                .address(),
            pte_address + mem::mb(2 * indices[2]) + mem::kb(4 * indices[3]) + page_offset
        );
    }

    #[test]
    fn x64_check_entry() {
        let mmu = get_mmu_spec();

        let pte_address = 1.into();
        assert!(mmu.check_entry(pte_address, 0));
        assert!(mmu.check_entry(pte_address, 1));
        assert!(mmu.check_entry(pte_address, 2));
        assert!(mmu.check_entry(pte_address, 3));
        assert!(mmu.check_entry(pte_address, 4));

        let pte_address = Address::null();
        assert!(mmu.check_entry(pte_address, 0));
        assert!(!mmu.check_entry(pte_address, 3));
    }

    #[test]
    fn x64_is_final_mapping() {
        let mmu = get_mmu_spec();

        let pte_address = (1 << 7).into();
        assert!(!mmu.is_final_mapping(pte_address, 0));
        assert!(!mmu.is_final_mapping(pte_address, 1));
        assert!(mmu.is_final_mapping(pte_address, 2));
        assert!(mmu.is_final_mapping(pte_address, 3));
        assert!(mmu.is_final_mapping(pte_address, 4));

        let pte_address = Address::null();
        assert!(!mmu.is_final_mapping(pte_address, 0));
        assert!(!mmu.is_final_mapping(pte_address, 1));
        assert!(!mmu.is_final_mapping(pte_address, 2));
        assert!(!mmu.is_final_mapping(pte_address, 3));
        assert!(mmu.is_final_mapping(pte_address, 4));
    }
}
