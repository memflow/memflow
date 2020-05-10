use super::{TLBCache, TLBEntry};
use crate::types::{Address, Length, Page, PhysicalAddress};

pub use coarsetime::{Duration, Instant};

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

#[derive(Clone, Copy)]
pub struct CachedEntry {
    dtb: Address,
    virt_page: Address,
    phys_page: Page,
}

impl CachedEntry {
    const INVALID: CachedEntry = CachedEntry {
        dtb: Address::INVALID,
        virt_page: Address::INVALID,
        phys_page: Page::INVALID,
    };
}

#[derive(Clone)]
pub struct TimedTLB {
    entries: Box<[CachedEntry]>,
    time: Box<[Instant]>,
    cache_time: Duration,
}

impl TimedTLB {
    pub fn new(size: Length, duration: Duration) -> Self {
        Self {
            entries: vec![CachedEntry::INVALID; size.as_usize()].into_boxed_slice(),
            time: vec![Instant::now(); size.as_usize()].into_boxed_slice(),
            cache_time: duration,
        }
    }

    fn get_cache_index(&self, page_addr: Address) -> usize {
        let mut hasher = DefaultHasher::new();
        page_addr.as_u64().hash(&mut hasher);
        (hasher.finish() % (self.entries.len() as u64)) as usize
    }
}

impl TLBCache for TimedTLB {
    fn try_entry(&mut self, dtb: Address, addr: Address, page_size: Length) -> Option<TLBEntry> {
        let page_address = addr.as_page_aligned(page_size);
        let idx = self.get_cache_index(page_address);
        let entry = self.entries[idx];
        if entry.dtb == dtb
            && entry.virt_page == page_address
            && entry.phys_page.page_base != Address::INVALID
        {
            if self.time[idx].elapsed() < self.cache_time {
                Some(TLBEntry {
                    dtb,
                    virt_addr: addr,
                    phys_addr: PhysicalAddress {
                        address: entry.phys_page.page_base + (addr - page_address),
                        page: Some(entry.phys_page),
                    },
                })
            } else {
                self.entries[idx].dtb = Address::INVALID;
                None
            }
        } else {
            None
        }
    }

    fn cache_entry(&mut self, dtb: Address, in_addr: Address, out_page: Page, page_size: Length) {
        let idx = self.get_cache_index(in_addr.as_page_aligned(page_size));
        self.entries[idx] = CachedEntry {
            dtb,
            virt_page: in_addr.as_page_aligned(page_size),
            phys_page: out_page,
        };
        self.time[idx] = Instant::now();
    }
}
