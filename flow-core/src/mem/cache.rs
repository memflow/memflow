pub mod cached_memory_access;
pub mod timed_cache;

pub use cached_memory_access::*;
pub use timed_cache::*;

use crate::error::Result;
use crate::Address;

bitflags! {
    pub struct PageType: u8 {
        const UNKNOWN = 0b0000_0001;
        const PAGE_TABLE = 0b0000_0010;
        const WRITEABLE = 0b0000_0100;
        const READ_ONLY = 0b0000_1000;
    }
}

impl PageType {
    pub fn from_writeable_bit(writeable: bool) -> Self {
        if writeable {
            PageType::WRITEABLE
        } else {
            PageType::READ_ONLY
        }
    }
}

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
    fn cached_page(&mut self, addr: Address, page_type: PageType) -> Result<CacheEntry>;
    fn validate_page(&mut self, addr: Address, page_type: PageType);
    fn invalidate_page(&mut self, addr: Address, page_type: PageType);
}
