use std::prelude::v1::*;

use super::CacheValidator;
use crate::architecture::{ArchitectureObj, ScopedVirtualTranslate};
use crate::error::{Error, Result};
use crate::types::{Address, PhysicalAddress};

#[derive(Clone, Copy)]
pub struct TLBEntry {
    pub pt_index: usize,
    pub virt_addr: Address,
    pub phys_addr: PhysicalAddress,
}

impl TLBEntry {
    pub const fn create_invalid() -> Self {
        Self {
            pt_index: !0,
            virt_addr: Address::INVALID,
            phys_addr: PhysicalAddress::INVALID,
        }
    }
}

#[derive(Clone, Copy)]
pub struct CachedEntry {
    pt_index: usize,
    virt_page: Address,
    phys_page: PhysicalAddress,
}

impl CachedEntry {
    const INVALID: CachedEntry = CachedEntry {
        pt_index: !0,
        virt_page: Address::INVALID,
        phys_page: PhysicalAddress::INVALID,
    };
}

#[derive(Clone)]
pub struct TLBCache<T> {
    entries: Box<[CachedEntry]>,
    pub validator: T,
}

impl<T: CacheValidator> TLBCache<T> {
    pub fn new(size: usize, mut validator: T) -> Self {
        validator.allocate_slots(size);

        Self {
            entries: vec![CachedEntry::INVALID; size].into_boxed_slice(),
            validator,
        }
    }

    #[inline]
    fn get_cache_index(&self, page_addr: Address, page_size: usize) -> usize {
        ((page_addr.as_u64() / (page_size as u64)) % (self.entries.len() as u64)) as usize
    }

    #[inline]
    pub fn is_read_too_long(&self, arch: ArchitectureObj, size: usize) -> bool {
        size / arch.page_size() > self.entries.len()
    }

    #[inline]
    pub fn try_entry<D: ScopedVirtualTranslate>(
        &self,
        translator: &D,
        addr: Address,
        arch: ArchitectureObj,
    ) -> Option<Result<TLBEntry>> {
        let pt_index = translator.translation_table_id(addr);
        let page_size = arch.page_size();
        let page_address = addr.as_page_aligned(page_size);
        let idx = self.get_cache_index(page_address, page_size);
        let entry = self.entries[idx];
        if entry.pt_index == pt_index
            && entry.virt_page == page_address
            && self.validator.is_slot_valid(idx)
        {
            if entry.phys_page.is_valid() && entry.phys_page.has_page() {
                Some(Ok(TLBEntry {
                    pt_index,
                    virt_addr: addr,
                    // TODO: this should be aware of huge pages
                    phys_addr: PhysicalAddress::with_page(
                        entry.phys_page.address().as_page_aligned(page_size)
                            + (addr - page_address),
                        entry.phys_page.page_type(),
                        page_size,
                    ),
                }))
            } else {
                Some(Err(Error::VirtualTranslate))
            }
        } else {
            None
        }
    }

    #[inline]
    pub fn cache_entry<D: ScopedVirtualTranslate>(
        &mut self,
        translator: &D,
        in_addr: Address,
        out_page: PhysicalAddress,
        arch: ArchitectureObj,
    ) {
        let pt_index = translator.translation_table_id(in_addr);
        let page_size = arch.page_size();
        let idx = self.get_cache_index(in_addr.as_page_aligned(page_size), page_size);
        self.entries[idx] = CachedEntry {
            pt_index,
            virt_page: in_addr.as_page_aligned(page_size),
            phys_page: out_page,
        };
        self.validator.validate_slot(idx);
    }

    #[inline]
    pub fn cache_invalid_if_uncached<D: ScopedVirtualTranslate>(
        &mut self,
        translator: &D,
        in_addr: Address,
        invalid_len: usize,
        arch: ArchitectureObj,
    ) {
        let pt_index = translator.translation_table_id(in_addr);
        let page_size = arch.page_size();
        let page_addr = in_addr.as_page_aligned(page_size);
        let end_addr = (in_addr + invalid_len + 1).as_page_aligned(page_size);

        for i in (page_addr.as_u64()..end_addr.as_u64())
            .step_by(page_size)
            .take(self.entries.len())
        {
            let cur_page = Address::from(i);
            let idx = self.get_cache_index(cur_page, page_size);

            let entry = &mut self.entries[idx];
            if entry.pt_index == !0
                || !entry.phys_page.is_valid()
                || !self.validator.is_slot_valid(idx)
            {
                entry.pt_index = pt_index;
                entry.virt_page = cur_page;
                entry.phys_page = PhysicalAddress::INVALID;
                self.validator.validate_slot(idx);
            }
        }
    }
}
