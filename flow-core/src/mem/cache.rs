pub mod cached_memory_access;
pub mod cached_vat;
pub mod timed_cache;
pub mod timed_tlb;

pub use cached_memory_access::*;
pub use cached_vat::*;
pub use timed_cache::*;
pub use timed_tlb::*;

use crate::types::{Address, Length, Page, PageType, PhysicalAddress};

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

pub trait TLBCache {
    fn try_entry(&mut self, dtb: Address, addr: Address, page_size: Length) -> Option<TLBEntry>;
    fn cache_entry(&mut self, dtb: Address, in_addr: Address, out_page: Page, page_size: Length);
}
