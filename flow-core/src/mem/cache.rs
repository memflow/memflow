pub mod cached_memory_access;
pub mod timed_cache;

pub use cached_memory_access::*;
pub use timed_cache::*;

use crate::address::{Address, Length, PageType};

// TODO: overhaul this mess, we should not throw with mutable memory around like this
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

pub trait PageCache {
    fn page_size(&self) -> Length;
    fn is_cached_page_type(&self, page_type: PageType) -> bool;

    fn cached_page_mut(&mut self, addr: Address) -> CacheEntry;

    fn validate_page(&mut self, addr: Address, page_type: PageType);
    fn invalidate_page(&mut self, addr: Address, page_type: PageType);
}
