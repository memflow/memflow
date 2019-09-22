#[cfg(feature = "x64")]
pub mod x64;

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
