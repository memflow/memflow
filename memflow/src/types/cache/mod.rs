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
    // TODO: better docs

    /// Allocate the required amount of slots used for validation
    fn allocate_slots(&mut self, slot_count: usize);

    /// Invoked once per Read/Write so internal state can be updated if necessary.
    ///
    /// This is an optimization so things like `std::time::Instant` only need to be computed once.
    fn update_validity(&mut self) {
        // no-op
    }

    /// Checks wether or not the given slot is valid.
    fn is_slot_valid(&self, slot_id: usize) -> bool;

    /// Callback from the cache implementation when a page is cached
    /// and the slot should become valid.
    fn validate_slot(&mut self, slot_id: usize);

    /// Callback from the caching implementation to mark a slot as invalid.
    ///
    /// This can happen if two different cache entries fall into the same slot id.
    fn invalidate_slot(&mut self, slot_id: usize);
}
