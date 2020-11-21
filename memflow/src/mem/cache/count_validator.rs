/*!
Validators are used when working with caches and determine for how long
a specific cache entry stays valid.

This validator limits the cache time based on an actual time instant.
Internally it uses the [coarsetime](https://docs.rs/coarsetime/0.1.14/coarsetime/) crate as a less
computation intensive alternative for [std::time](https://doc.rust-lang.org/std/time/index.html).
Therefor the Duration has to be converted (e.g. via the .into() trait) when constructing this validator.

The default implementation will set the cache time to 1 second.
*/
use std::prelude::v1::*;

use super::CacheValidator;

/// Validator for limiting the cache time based on a time `Instant`
///
/// # Remarks
///
/// This validator is only available when being compiled with `std`.
/// When using `no_std` you might want to use another validator.
/// TODO: add other validators here
#[derive(Clone)]
pub struct CountCacheValidator {
    count: Vec<usize>,
    valid_count: usize,
    last_count: usize,
}

/// Creates a validator with a cache timeout of 1 second.
impl Default for CountCacheValidator {
    fn default() -> Self {
        Self::new(10)
    }
}

impl CountCacheValidator {
    /// Creates a new CountCacheValidator with a customizable valid count.
    ///
    /// Valid count is increased on every memory operation by the validator users.
    ///
    /// # Examples:
    /// ```
    /// use memflow::mem::{CacheValidator, CountCacheValidator};
    ///
    /// let mut validator = CountCacheValidator::new(100);
    ///
    /// validator.allocate_slots(1);
    ///
    /// assert!(!validator.is_slot_valid(0));
    /// validator.validate_slot(0);
    ///
    /// // For a hundred times the slot should stay valid
    /// for _ in 0..100 {
    ///     assert!(validator.is_slot_valid(0));
    ///     validator.update_validity();
    /// }
    ///
    /// // At this point it should become invalid
    /// assert!(!validator.is_slot_valid(0));
    /// ```
    pub fn new(valid_count: usize) -> Self {
        Self {
            count: vec![],
            valid_count,
            last_count: 0,
        }
    }
}

impl CacheValidator for CountCacheValidator {
    #[inline]
    fn allocate_slots(&mut self, slot_count: usize) {
        self.count
            .resize(slot_count, self.last_count.wrapping_sub(self.valid_count));
    }

    #[inline]
    fn update_validity(&mut self) {
        self.last_count = self.last_count.wrapping_add(1);
    }

    #[inline]
    fn is_slot_valid(&self, slot_id: usize) -> bool {
        self.last_count.wrapping_sub(self.count[slot_id]) < self.valid_count
    }

    #[inline]
    fn validate_slot(&mut self, slot_id: usize) {
        self.count[slot_id] = self.last_count;
    }

    #[inline]
    fn invalidate_slot(&mut self, slot_id: usize) {
        self.count[slot_id] = self.last_count - self.valid_count
    }
}
