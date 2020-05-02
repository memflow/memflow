use super::{CacheEntry, PageCache, PageType};
use crate::address::{Address, Length};
use crate::{Error, Result};

use coarsetime::{Duration, Instant};

// the set page_size must be smaller than the target's page_size, otherwise this would trigger UB
#[derive(Clone)]
pub struct TimedCache {
    address: Box<[Address]>,
    time: Box<[Instant]>,
    cache: Box<[u8]>,
    cache_time: Duration,
    page_size: Length,
    page_mask: PageType,
}

impl TimedCache {
    pub fn new(
        cache_time_millis: u64,
        cache_size: usize,
        page_size: Length,
        page_mask: PageType,
    ) -> Self {
        Self {
            address: vec![(!0_u64).into(); cache_size].into_boxed_slice(),
            time: vec![Instant::now(); cache_size].into_boxed_slice(),
            cache: vec![0; cache_size * page_size.as_usize()].into_boxed_slice(),
            cache_time: Duration::from_millis(cache_time_millis),
            page_size,
            page_mask,
        }
    }

    fn page_index(&self, addr: Address) -> usize {
        (addr.as_page_aligned(self.page_size).as_usize() / self.page_size.as_usize())
            % self.address.len()
    }

    fn page_and_info_from_index(&mut self, idx: usize) -> (&mut [u8], &mut Address, &mut Instant) {
        let start = self.page_size.as_usize() * idx;
        (
            &mut self.cache[start..(start + self.page_size.as_usize())],
            &mut self.address[idx],
            &mut self.time[idx],
        )
    }

    fn page_from_index(&mut self, idx: usize) -> &mut [u8] {
        let start = self.page_size.as_usize() * idx;
        &mut self.cache[start..(start + self.page_size.as_usize())]
    }

    fn try_page_with_time(
        &mut self,
        addr: Address,
        time: Instant,
    ) -> std::result::Result<&mut [u8], (&mut [u8], &mut Address, &mut Instant)> {
        let page_index = self.page_index(addr);
        if self.address[page_index] == addr.as_page_aligned(self.page_size)
            && time.duration_since(self.time[page_index]) <= self.cache_time
        {
            Ok(self.page_from_index(page_index))
        } else {
            Err(self.page_and_info_from_index(page_index))
        }
    }
}

impl Default for TimedCache {
    fn default() -> Self {
        Self::new(
            1000,
            0x200,
            Length::from_kb(4),
            PageType::PAGE_TABLE | PageType::READ_ONLY,
        )
    }
}

impl PageCache for TimedCache {
    fn cached_page(&mut self, addr: Address, page_type: PageType) -> Result<CacheEntry> {
        // TODO: optimize internal functions
        if (self.page_mask & page_type).is_empty() {
            Err(Error::new("page is not cached"))
        } else {
            let page_size = self.page_size;
            let aligned_addr = addr.as_page_aligned(page_size);
            match self.try_page_with_time(addr, Instant::now()) {
                Ok(page) => Ok(CacheEntry {
                    valid: true,
                    address: aligned_addr,
                    buf: page,
                }),
                Err((page, _, _)) => Ok(CacheEntry {
                    valid: false,
                    address: aligned_addr,
                    buf: page,
                }),
            }
        }
    }

    fn validate_page(&mut self, addr: Address, page_type: PageType) {
        if !(self.page_mask & page_type).is_empty() {
            let idx = self.page_index(addr);
            let aligned_addr = addr.as_page_aligned(self.page_size);
            let page_info = self.page_and_info_from_index(idx);
            *page_info.1 = aligned_addr;
            *page_info.2 = Instant::now();
        }
    }

    fn invalidate_page(&mut self, addr: Address, page_type: PageType) {
        if !(self.page_mask & page_type).is_empty() {
            let idx = self.page_index(addr);
            let page_info = self.page_and_info_from_index(idx);
            *page_info.1 = Address::null();
        }
    }
}
