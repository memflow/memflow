use super::CacheValidator;
use crate::types::{Address, Length, PhysicalAddress};

#[derive(Clone, Copy)]
pub struct TLBEntry {
    pub dtb: Address,
    pub virt_addr: Address,
    pub phys_addr: PhysicalAddress,
}

impl TLBEntry {
    pub const fn create_invalid() -> Self {
        Self {
            dtb: Address::INVALID,
            virt_addr: Address::INVALID,
            phys_addr: PhysicalAddress::INVALID,
        }
    }
}

#[derive(Clone, Copy)]
pub struct CachedEntry {
    dtb: Address,
    virt_page: Address,
    phys_page: PhysicalAddress,
}

impl CachedEntry {
    const INVALID: CachedEntry = CachedEntry {
        dtb: Address::INVALID,
        virt_page: Address::INVALID,
        phys_page: PhysicalAddress::INVALID,
    };
}

#[derive(Clone)]
pub struct TLBCache<T: CacheValidator> {
    entries: Box<[CachedEntry]>,
    pub validator: T,
}

impl<T: CacheValidator> TLBCache<T> {
    pub fn new(size: Length, mut validator: T) -> Self {
        validator.allocate_slots(size.as_usize());

        Self {
            entries: vec![CachedEntry::INVALID; size.as_usize()].into_boxed_slice(),
            validator,
        }
    }

    #[inline]
    fn get_cache_index(&self, page_addr: Address, page_size: Length) -> usize {
        ((page_addr.as_u64() / page_size.as_u64()) % (self.entries.len() as u64)) as usize
    }

    #[inline]
    pub fn try_entry_ref(
        &self,
        dtb: Address,
        addr: Address,
        page_size: Length,
    ) -> Option<TLBEntry> {
        let page_address = addr.as_page_aligned(page_size);
        let idx = self.get_cache_index(page_address, page_size);
        let entry = self.entries[idx];
        if entry.dtb == dtb
            && entry.virt_page == page_address
            && entry.phys_page.is_valid()
            && entry.phys_page.has_page()
            && self.validator.is_slot_valid(idx)
        {
            Some(TLBEntry {
                dtb,
                virt_addr: addr,
                // TODO: this should be aware of huge pages
                phys_addr: PhysicalAddress::with_page(
                    entry.phys_page.address().as_page_aligned(page_size) + (addr - page_address),
                    entry.phys_page.page_type(),
                    page_size,
                ),
            })
        } else {
            None
        }
    }

    #[inline]
    pub fn try_entry(
        &mut self,
        dtb: Address,
        addr: Address,
        page_size: Length,
    ) -> Option<TLBEntry> {
        let page_address = addr.as_page_aligned(page_size);
        let idx = self.get_cache_index(page_address, page_size);
        let entry = self.entries[idx];
        if entry.dtb == dtb
            && entry.virt_page == page_address
            && entry.phys_page.is_valid()
            && entry.phys_page.has_page()
        {
            if self.validator.is_slot_valid(idx) {
                Some(TLBEntry {
                    dtb,
                    virt_addr: addr,
                    // TODO: this should be aware of huge pages
                    phys_addr: PhysicalAddress::with_page(
                        entry.phys_page.address().as_page_aligned(page_size)
                            + (addr - page_address),
                        entry.phys_page.page_type(),
                        page_size,
                    ),
                })
            } else {
                self.entries[idx].dtb = Address::INVALID;
                None
            }
        } else {
            None
        }
    }

    #[inline]
    pub fn cache_entry(
        &mut self,
        dtb: Address,
        in_addr: Address,
        out_page: PhysicalAddress,
        page_size: Length,
    ) {
        let idx = self.get_cache_index(in_addr.as_page_aligned(page_size), page_size);
        self.entries[idx] = CachedEntry {
            dtb,
            virt_page: in_addr.as_page_aligned(page_size),
            phys_page: out_page,
        };
        self.validator.invalidate_slot(idx);
    }
}
