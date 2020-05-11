use super::{CacheValidator, PageType};
use crate::architecture::Architecture;
use crate::types::{Address, Length};
use std::alloc::{alloc_zeroed, Layout};

pub struct CacheEntry<'a> {
    pub valid: bool,
    pub address: Address,
    pub buf: &'a mut [u8],
}

impl<'a> CacheEntry<'a> {
    pub fn with(valid: bool, address: Address, buf: &'a mut [u8]) -> Self {
        Self {
            valid,
            address,
            buf,
        }
    }

    pub fn is_valid(&self) -> bool {
        self.valid
    }

    pub fn address(&self) -> Address {
        self.address
    }
}

#[derive(Clone)]
pub struct PageCache<T: CacheValidator> {
    address: Box<[Address]>,
    cache: Box<[u8]>,
    page_size: Length,
    page_type_mask: PageType,
    pub validator: T,
}

impl<T: CacheValidator> PageCache<T> {
    pub fn new(
        arch: Architecture,
        size: Length,
        page_type_mask: PageType,
        mut validator: T,
    ) -> Self {
        let page_size = arch.page_size();
        let cache_entries = size.as_usize() / page_size.as_usize();

        let layout =
            Layout::from_size_align(cache_entries * page_size.as_usize(), page_size.as_usize())
                .unwrap();

        let cache = unsafe {
            Box::from_raw(std::slice::from_raw_parts_mut(
                alloc_zeroed(layout),
                layout.size(),
            ))
        };

        validator.allocate_slots(cache_entries);

        Self {
            address: vec![(!0_u64).into(); cache_entries].into_boxed_slice(),
            cache,
            page_size,
            page_type_mask,
            validator,
        }
    }

    fn page_index(&self, addr: Address) -> usize {
        (addr.as_page_aligned(self.page_size).as_usize() / self.page_size.as_usize())
            % self.address.len()
    }

    fn page_and_info_from_index(&mut self, idx: usize) -> (&mut [u8], &mut Address) {
        let start = self.page_size.as_usize() * idx;
        (
            &mut self.cache[start..(start + self.page_size.as_usize())],
            &mut self.address[idx],
        )
    }

    fn page_from_index(&mut self, idx: usize) -> &mut [u8] {
        let start = self.page_size.as_usize() * idx;
        &mut self.cache[start..(start + self.page_size.as_usize())]
    }

    fn try_page(
        &mut self,
        addr: Address,
    ) -> std::result::Result<&mut [u8], (&mut [u8], &mut Address)> {
        let page_index = self.page_index(addr);
        if self.address[page_index] == addr.as_page_aligned(self.page_size)
            && self.validator.is_slot_valid(page_index)
        {
            Ok(self.page_from_index(page_index))
        } else {
            Err(self.page_and_info_from_index(page_index))
        }
    }

    pub fn page_size(&self) -> Length {
        self.page_size
    }

    pub fn is_cached_page_type(&self, page_type: PageType) -> bool {
        self.page_type_mask.contains(page_type)
    }

    pub fn cached_page_mut(&mut self, addr: Address) -> CacheEntry {
        let page_size = self.page_size;
        let aligned_addr = addr.as_page_aligned(page_size);
        match self.try_page(addr) {
            Ok(page) => CacheEntry {
                valid: true,
                address: aligned_addr,
                buf: page,
            },
            Err((page, _)) => CacheEntry {
                valid: false,
                address: aligned_addr,
                buf: page,
            },
        }
    }

    pub fn validate_page(&mut self, addr: Address, page_type: PageType) {
        if self.page_type_mask.contains(page_type) {
            let idx = self.page_index(addr);
            let aligned_addr = addr.as_page_aligned(self.page_size);
            let page_info = self.page_and_info_from_index(idx);
            *page_info.1 = aligned_addr;
            self.validator.validate_slot(idx);
        }
    }

    pub fn invalidate_page(&mut self, addr: Address, page_type: PageType) {
        if self.page_type_mask.contains(page_type) {
            let idx = self.page_index(addr);
            let page_info = self.page_and_info_from_index(idx);
            *page_info.1 = Address::null();
            self.validator.invalidate_slot(idx);
        }
    }
}
