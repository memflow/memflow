pub mod cached_memory_access;
pub mod cached_vat;
pub mod page_cache;
pub mod timed_validator;
pub mod tlb_cache;

pub use cached_memory_access::*;
pub use cached_vat::*;
pub use page_cache::*;
pub use tlb_cache::*;

use crate::types::{Address, PageType};

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

pub trait CacheValidator {
    fn allocate_slots(&mut self, slot_count: usize);
    fn update_validity(&mut self);
    fn is_slot_valid(&mut self, slot_id: usize) -> bool;
    fn validate_slot(&mut self, slot_id: usize);
    fn invalidate_slot(&mut self, slot_id: usize);
}
