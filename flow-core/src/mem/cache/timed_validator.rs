use super::CacheValidator;
pub use coarsetime::{Duration, Instant};

#[derive(Clone)]
pub struct TimedCacheValidator {
    time: Vec<Instant>,
    valid_time: Duration,
    last_time: Instant,
}

impl TimedCacheValidator {
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
