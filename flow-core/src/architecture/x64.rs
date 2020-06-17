use super::ArchMMUSpec;
use crate::architecture::Endianess;
use crate::types::Length;

pub fn bits() -> u8 {
    64
}

pub fn endianess() -> Endianess {
    Endianess::LittleEndian
}

pub fn len_addr() -> Length {
    Length::from(8)
}

pub fn get_mmu_spec() -> ArchMMUSpec {
    ArchMMUSpec {
        virtual_address_splits: &[9, 9, 9, 9, 12],
        valid_final_page_steps: &[2, 3, 4],
        address_space_bits: 52,
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

#[cfg(test)]
mod tests {
    use super::super::mmu_spec::masks::*;
    use super::get_mmu_spec;
    use crate::types::{Address, Length, PageType};

    #[test]
    fn x64_pte_bitmasks() {
        let mmu = get_mmu_spec();
        let mask_addr = Address::invalid();
        assert_eq!(mmu.pte_addr_mask(mask_addr, 0), make_bit_mask(12, 51));
        assert_eq!(mmu.pte_addr_mask(mask_addr, 1), make_bit_mask(12, 51));
        assert_eq!(mmu.pte_addr_mask(mask_addr, 2), make_bit_mask(12, 51));
        assert_eq!(mmu.pte_addr_mask(mask_addr, 3), make_bit_mask(12, 51));
    }

    #[test]
    fn x64_split_count() {
        assert_eq!(get_mmu_spec().split_count(), 5);
    }

    #[test]
    fn x64_pte_leaf_size() {
        let mmu = get_mmu_spec();
        assert_eq!(mmu.pt_leaf_size(0), Length::from_kb(4));
    }

    #[test]
    fn x64_page_size_level() {
        let mmu = get_mmu_spec();
        assert_eq!(mmu.page_size_level(1), Length::from_kb(4));
        assert_eq!(mmu.page_size_level(2), Length::from_mb(2));
        assert_eq!(mmu.page_size_level(3), Length::from_gb(1));
    }

    #[test]
    fn x64_page_size_step() {
        let mmu = get_mmu_spec();
        assert_eq!(mmu.page_size_step(2), Length::from_gb(1));
        assert_eq!(mmu.page_size_step(3), Length::from_mb(2));
        assert_eq!(mmu.page_size_step(4), Length::from_kb(4));
    }

    #[test]
    #[should_panic]
    #[cfg(debug_assertions)]
    fn x64_page_size_level_4() {
        let mmu = get_mmu_spec();
        assert_eq!(mmu.page_size_level(4), Length::from_gb(512));
    }

    #[test]
    #[should_panic]
    #[cfg(debug_assertions)]
    fn x64_page_size_level_5() {
        let mmu = get_mmu_spec();
        assert_eq!(mmu.page_size_level(5), Length::from_gb(512 * 512));
    }

    #[test]
    fn x64_vtop_step() {
        let mmu = get_mmu_spec();
        let indices = [145u64, 54, 64, 0];
        let virt_address = indices
            .iter()
            .rev()
            .enumerate()
            .fold(0, |state, (lvl, idx)| state | (idx << (12 + 9 * lvl)))
            .into();
        let pte_address = Address::from(Length::from_kb(4 * 45));
        assert_eq!(
            mmu.vtop_step(pte_address, virt_address, 0),
            pte_address + Length::from(indices[0] * 8)
        );
        assert_eq!(
            mmu.vtop_step(pte_address, virt_address, 1),
            pte_address + Length::from(indices[1] * 8)
        );
        assert_eq!(
            mmu.vtop_step(pte_address, virt_address, 2),
            pte_address + Length::from(indices[2] * 8)
        );
        assert_eq!(
            mmu.vtop_step(pte_address, virt_address, 3),
            pte_address + Length::from(indices[3] * 8)
        );
    }

    #[test]
    fn x64_get_phys_page() {
        let mmu = get_mmu_spec();
        let indices = [145u64, 54, 64, 21];
        let page_offset = 1243;
        let virt_address = indices
            .iter()
            .rev()
            .enumerate()
            .fold(page_offset, |state, (lvl, idx)| {
                state | (idx << (12 + 9 * lvl))
            })
            .into();
        let pte_address = Address::from(Length::from_gb(57));

        assert_eq!(
            mmu.get_phys_page(pte_address, virt_address, 4).page_type(),
            PageType::READ_ONLY
        );
        assert_eq!(
            mmu.get_phys_page(pte_address, virt_address, 4).page_size(),
            Length::from_kb(4)
        );
        assert_eq!(
            mmu.get_phys_page(pte_address, virt_address, 2).page_base(),
            pte_address
        );

        assert_eq!(
            mmu.get_phys_page(pte_address, virt_address, 4).address(),
            pte_address + Length::from(page_offset)
        );
        assert_eq!(
            mmu.get_phys_page(pte_address, virt_address, 3).address(),
            pte_address + Length::from_kb(4 * indices[3]) + Length::from(page_offset)
        );
        assert_eq!(
            mmu.get_phys_page(pte_address, virt_address, 2).address(),
            pte_address
                + Length::from_mb(2 * indices[2])
                + Length::from_kb(4 * indices[3])
                + Length::from(page_offset)
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
