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
use coarsetime::{Duration, Instant};

/// Validator for limiting the cache time based on a time `Instant`
///
/// # Remarks
///
/// This validator is only available when being compiled with `std`.
/// When using `no_std` you might want to use another validator.
/// TODO: add other validators here
#[derive(Clone)]
pub struct TimedCacheValidator {
    time: Vec<Instant>,
    valid_time: Duration,
    last_time: Instant,
}

/// Creates a validator with a cache timeout of 1 second.
impl Default for TimedCacheValidator {
    fn default() -> Self {
        Self::new(Duration::from_millis(1000))
    }
}

impl TimedCacheValidator {
    /// Creates a new TimedCacheValidator with a customizable Duration.
    ///
    /// # Examples:
    /// ```
    /// use std::time::Duration;
    /// use memflow::mem::TimedCacheValidator;
    ///
    /// let _ = TimedCacheValidator::new(Duration::from_millis(5000).into());
    /// ```
    pub fn new(valid_time: Duration) -> Self {
        Self {
            time: vec![],
            valid_time,
            last_time: Instant::now(),
        }
    }
}

impl CacheValidator for TimedCacheValidator {
    #[inline]
    fn allocate_slots(&mut self, slot_count: usize) {
        self.time
            .resize(slot_count, self.last_time - self.valid_time);
    }

    #[inline]
    fn update_validity(&mut self) {
        self.last_time = Instant::now()
    }

    #[inline]
    fn is_slot_valid(&self, slot_id: usize) -> bool {
        self.last_time.duration_since(self.time[slot_id]) <= self.valid_time
    }

    #[inline]
    fn validate_slot(&mut self, slot_id: usize) {
        self.time[slot_id] = self.last_time;
    }

    #[inline]
    fn invalidate_slot(&mut self, slot_id: usize) {
        self.time[slot_id] = self.last_time - self.valid_time
    }
}
