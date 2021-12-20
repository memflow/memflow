#[cfg(feature = "std")]
pub mod timed_validator;

pub mod count_validator;

#[cfg(feature = "std")]
#[doc(hidden)]
pub use timed_validator::*;

#[doc(hidden)]
pub use count_validator::*;

#[cfg(feature = "std")]
pub type DefaultCacheValidator = TimedCacheValidator;
#[cfg(not(feature = "std"))]
pub type DefaultCacheValidator = CountCacheValidator;

/// Validators are used when working with caches and determine for how long
/// a specific cache entry stays valid.
pub trait CacheValidator
where
    Self: Send,
{
    fn allocate_slots(&mut self, slot_count: usize);
    fn update_validity(&mut self);
    fn is_slot_valid(&self, slot_id: usize) -> bool;
    fn validate_slot(&mut self, slot_id: usize);
    fn invalidate_slot(&mut self, slot_id: usize);
}
