use flow_core::mem::PhysicalMemory;

// TODO: how do we abstract different architectures?
pub trait VirtualAddressTranslation {
	fn x64_virt_to_phys(cr3: u64, addr: u64) -> u64;
}

// bit mask macros
const fn make_bit_mask(a: u32, b: u32) -> u64 {
	(0xffffffffffffffff >> (63 - b)) & !(((1 as u64) << a) - 1)
}

macro_rules! get_bit {
	($a:expr, $b:expr) => {
		($a & ((1 as u64) << $b)) != 0
	};
}

// page test macros
macro_rules! is_large_page {
	($a:expr) => {
		get_bit!($a, 7)
	};
}

macro_rules! is_transition_page {
	($a:expr) => {
		get_bit!($a, 11)
	};
}

macro_rules! is_prototype_page {
	($a:expr) => {
		get_bit!($a, 10)
	};
}

//#define CHECK_ENTRY(entry) (GET_BIT(entry, 0) ? 1 : ((IS_TRANSITION_PAGE(entry) && !(IS_PROTOTYPE_PAGE(entry))) ? 1 : 0))

// TODO: write tests for these macros
// pagetable indizes
macro_rules! pml4_index_bits {
	($a:expr) => {
		($a & make_bit_mask!(39, 47)) >> 36
	};
}

macro_rules! pdpte_index_bits {
	($a:expr) => {
		($a & make_bit_mask!(30, 38)) >> 27
	};
}

macro_rules! pd_index_bits {
	($a:expr) => {
		($a & make_bit_mask!(21, 29)) >> 18
	};
}

macro_rules! pt_index_bits {
	($a:expr) => {
		($a & make_bit_mask!(12, 20)) >> 9
	};
}

impl<T: PhysicalMemory> VirtualAddressTranslation for T {
	fn x64_virt_to_phys(cr3: u64, addr: u64) -> u64 {
		let mask = make_bit_mask(12, 51);
		0
	}
}

/*
uint64_t
phys_process_manager::_virt_to_phys(uint64_t cr3, uint64_t addr) {
	uint64_t pml4e_addr, pml4e_value;
	if (this->_get_pml4e(addr, cr3, &pml4e_addr, &pml4e_value) < 0) {
#ifdef PHYS_DEBUG_VAT
		printf("failed to read pml4e\n");
#endif
		return 0;
	}

	if (!CHECK_ENTRY(pml4e_value)) {
#ifdef PHYS_DEBUG_VAT
		printf("invalid pmle entry\n");
#endif
		return 0;
	}

	uint64_t pdpte_addr, pdpte_value;
	if (this->_get_pdpte(addr, pml4e_value, &pdpte_addr, &pdpte_value) < 0) {
#ifdef PHYS_DEBUG_VAT
		printf("failed to read pdpte\n");
#endif
		return 0;
	}

	if (!CHECK_ENTRY(pdpte_value)) {
#ifdef PHYS_DEBUG_VAT
		printf("invalid pdpte entry\n");
#endif
		return 0;
	}

	if (IS_LARGE_PAGE(pdpte_value)) {
#ifdef PHYS_DEBUG_VAT
		printf("found 1gb page\n");
#endif
		return (pdpte_value & MAKE_BIT_MASK(30, 51)) | (addr & MAKE_BIT_MASK(0, 29));
	}

	uint64_t pgd_addr, pgd_value;
	if (this->_get_pde(addr, pdpte_value, &pgd_addr, &pgd_value) < 0) {
#ifdef PHYS_DEBUG_VAT
		printf("failed to read pde\n");
#endif
		return 0;
	}

	if (!CHECK_ENTRY(pgd_value)) {
#ifdef PHYS_DEBUG_VAT
		printf("invalid pgd entry\n");
#endif
		return 0;
	}

	if (IS_LARGE_PAGE(pgd_value)) {
#ifdef PHYS_DEBUG_VAT
		printf("found 2mb page\n");
#endif
		return (pgd_value & MAKE_BIT_MASK(21, 51)) | (addr & MAKE_BIT_MASK(0, 20));
	}

	uint64_t pte_addr, pte_value;
	if (this->_get_pte(addr, pgd_value, &pte_addr, &pte_value) < 0) {
#ifdef PHYS_DEBUG_VAT
		printf("failed to read pte\n");
#endif
		return 0;
	}

	if (!CHECK_ENTRY(pte_value)) {
#ifdef PHYS_DEBUG_VAT
		printf("invalid pte entry\n");
#endif
		return 0;
	}

#ifdef PHYS_DEBUG_VAT
	printf("found 4kb page\n");
#endif
	return (pte_value & MAKE_BIT_MASK(12, 51)) | (addr & MAKE_BIT_MASK(0, 11));
}

int
phys_process_manager::_get_pml4e(uint64_t addr, uint64_t cr3, uint64_t *pml4e_addr, uint64_t *pml4e_value) {
	*pml4e_addr = (cr3 & MAKE_BIT_MASK(12, 51)) | this->_get_pml4_index(addr);
	*pml4e_value = this->_read_phys<uint64_t>(*pml4e_addr);
	return *pml4e_value != 0;
}

int
phys_process_manager::_get_pdpte(uint64_t addr, uint64_t pml4e, uint64_t *pdpte_address, uint64_t *pdpte_value) {
	*pdpte_address = (pml4e & MAKE_BIT_MASK(12, 51)) | this->_get_pdpte_index(addr);
	*pdpte_value = this->_read_phys<uint64_t>(*pdpte_address);
	return *pdpte_value != 0;
}

int
phys_process_manager::_get_pde(uint64_t addr, uint64_t pdpte, uint64_t *pgd_addr, uint64_t *pgd_value) {
	*pgd_addr = (pdpte & MAKE_BIT_MASK(12, 51)) | this->_get_pd_index(addr);
	*pgd_value = this->_read_phys<uint64_t>(*pgd_addr);
	return *pgd_addr != 0;
}

int
phys_process_manager::_get_pte(uint64_t addr, uint64_t pgd, uint64_t *pte_address, uint64_t *pte_value) {
	*pte_address = (pgd & MAKE_BIT_MASK(12, 51)) | this->_get_pt_index(addr);
	*pte_value = this->_read_phys<uint64_t>(*pte_address);
	return *pte_address != 0;
}
*/

#[cfg(test)]
mod tests {
	use crate::make_bit_mask;

	#[test]
	fn test_make_bit_mask() {
		assert_eq!(make_bit_mask(0, 11), 0xfff);
		assert_eq!(make_bit_mask(12, 20), 0x1ff000);
		assert_eq!(make_bit_mask(21, 29), 0x3fe00000);
		assert_eq!(make_bit_mask(30, 38), 0x7fc0000000);
		assert_eq!(make_bit_mask(39, 47), 0xff8000000000);
		assert_eq!(make_bit_mask(12, 51), 0xffffffffff000);
	}

	#[test]
	fn test_get_bit() {
		//assert_eq!(make_bit_mask!(0, 11), 0xfff);
	}

	#[test]
	fn test_is_large_page() {
		assert_eq!(is_large_page!(0x00000000000000F0), true);
		assert_eq!(is_large_page!(0x0000000000000080), true);
		assert_eq!(is_large_page!(0x0000000000000070), false);
		assert_eq!(is_large_page!(0x0000000000000020), false);
	}

	#[test]
	fn test_is_transition_page() {
		assert_eq!(is_transition_page!(0x0000000000000F00), true);
		assert_eq!(is_transition_page!(0x0000000000000800), true);
		assert_eq!(is_transition_page!(0x0000000000000700), false);
		assert_eq!(is_transition_page!(0x0000000000000200), false);
	}

	#[test]
	fn test_is_prototype_page() {
		assert_eq!(is_prototype_page!(0x0000000000000F00), true);
		assert_eq!(is_prototype_page!(0x0000000000000800), false);
		assert_eq!(is_prototype_page!(0x0000000000000700), true);
		assert_eq!(is_prototype_page!(0x0000000000000200), false);
	}
}
