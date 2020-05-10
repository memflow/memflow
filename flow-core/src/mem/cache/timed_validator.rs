use super::CacheValidator;
pub use coarsetime::{Duration, Instant};

#[derive(Clone)]
pub struct TimedCacheValidator {
    time: Box<[Instant]>,
    valid_time: Duration,
}

impl TimedCacheValidator {
    pub fn new(valid_time: Duration) -> Self {
        Self {
            time: Box::new([]),
            valid_time,
        }
    }
}

impl CacheValidator for TimedCacheValidator {
    fn allocate_slots(&mut self, slot_count: usize) {
        self.time = vec![Instant::recent() - self.valid_time; slot_count].into_boxed_slice();
    }

    fn is_slot_valid(&mut self, slot_id: usize) -> bool {
        self.time[slot_id].elapsed() < self.valid_time
    }

    fn validate_slot(&mut self, slot_id: usize) {
        self.time[slot_id] = Instant::now()
    }

    fn invalidate_slot(&mut self, slot_id: usize) {
        self.time[slot_id] = Instant::recent() - self.valid_time
    }
}
