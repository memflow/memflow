pub mod cached_memory_access;
pub mod cached_vat;
pub mod timed_validator;

mod page_cache;
mod tlb_cache;

pub use cached_memory_access::*;
pub use cached_vat::*;
pub use timed_validator::*;

use crate::types::PageType;

pub trait CacheValidator {
    fn allocate_slots(&mut self, slot_count: usize);
    fn update_validity(&mut self);
    fn is_slot_valid(&self, slot_id: usize) -> bool;
    fn validate_slot(&mut self, slot_id: usize);
    fn invalidate_slot(&mut self, slot_id: usize);
}
