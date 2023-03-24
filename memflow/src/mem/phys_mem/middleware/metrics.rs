use ::log::info;
use ::std::{collections::VecDeque, time::Instant};

use crate::mem::{
    PhysicalMemory, PhysicalMemoryMapping, PhysicalMemoryMetadata, PhysicalReadMemOps,
    PhysicalWriteMemOps,
};
use crate::{error::Result, mem::MemOps};

/// The metrics middleware collects metrics data (latency and number of bytes) for all read and write operations.
/// Additionally metrics are outputted via `::log::info` in regular intervals.
///
/// Since this middleware implements [`PhysicalMemory`] it can be used as a replacement
/// in all structs and functions that require the [`PhysicalMemory`] trait.
pub struct PhysicalMemoryMetrics<T> {
    mem: T,
    reads: MemOpsHistory,
    last_read_info: Instant,
    writes: MemOpsHistory,
    last_write_info: Instant,
}

impl<T> Clone for PhysicalMemoryMetrics<T>
where
    T: Clone,
{
    fn clone(&self) -> Self {
        Self {
            mem: self.mem.clone(),
            reads: self.reads.clone(),
            last_read_info: Instant::now(),
            writes: self.writes.clone(),
            last_write_info: Instant::now(),
        }
    }
}

impl<T: PhysicalMemory> PhysicalMemoryMetrics<T> {
    /// Constructs a new middleware.
    pub fn new(mem: T) -> Self {
        // TODO: configurable number of samples?
        Self {
            mem,
            reads: MemOpsHistory::new(0..100, 1.0),
            last_read_info: Instant::now(),
            writes: MemOpsHistory::new(0..100, 1.0),
            last_write_info: Instant::now(),
        }
    }

    /// Consumes self and returns the containing memory object.
    ///
    /// This function can be useful in case the ownership over the memory object has been given to the cache
    /// when it was being constructed.
    /// It will destroy the `self` and return back the ownership of the underlying memory object.
    ///
    /// # Examples
    /// ```
    /// # const MAGIC_VALUE: u64 = 0x23bd_318f_f3a3_5821;
    /// use memflow::architecture::x86::x64;
    /// use memflow::mem::{PhysicalMemory, PhysicalMemoryMetrics, MemoryView};
    ///
    /// fn build<T: PhysicalMemory>(mem: T) -> T {
    ///     let mut middleware = PhysicalMemoryMetrics::new(mem);
    ///
    ///     // use the middleware...
    ///     let value: u64 = middleware.phys_view().read(0.into()).unwrap();
    ///     assert_eq!(value, MAGIC_VALUE);
    ///
    ///     // retrieve ownership of mem and return it back
    ///     middleware.into_inner()
    /// }
    /// # use memflow::dummy::DummyMemory;
    /// # use memflow::types::size;
    /// # let mut mem = DummyMemory::new(size::mb(4));
    /// # mem.phys_write(0.into(), &MAGIC_VALUE).unwrap();
    /// # build(mem);
    /// ```
    pub fn into_inner(self) -> T {
        self.mem
    }
}

// forward PhysicalMemory trait fncs
impl<T: PhysicalMemory> PhysicalMemory for PhysicalMemoryMetrics<T> {
    #[inline]
    fn phys_read_raw_iter(
        &mut self,
        MemOps { inp, out_fail, out }: PhysicalReadMemOps,
    ) -> Result<()> {
        let mut number_of_bytes = 0;
        let iter = inp.inspect(|e| number_of_bytes += e.2.len());

        let start_time = Instant::now();

        let mem = &mut self.mem;
        let result = MemOps::with_raw(iter, out, out_fail, |data| mem.phys_read_raw_iter(data));

        self.reads
            .add(start_time.elapsed().as_secs_f64(), number_of_bytes);

        //if self.reads.total_count() % 10000 == 0 {
        if self.last_read_info.elapsed().as_secs_f64() >= 1f64 {
            info!(
                "Read Metrics: reads_per_second={} average_latency={:.4}ms; average_bytes={}; bytes_per_second={}",
                self.reads.len(),
                self.reads.average_latency().unwrap_or_default() * 1000f64,
                self.reads.average_bytes().unwrap_or_default(),
                self.reads.bandwidth().unwrap_or_default(),
            );
            self.last_read_info = Instant::now();
        }

        result
    }

    #[inline]
    fn phys_write_raw_iter(
        &mut self,
        MemOps { inp, out_fail, out }: PhysicalWriteMemOps,
    ) -> Result<()> {
        let mut number_of_bytes = 0;
        let iter = inp.inspect(|e| number_of_bytes += e.2.len());

        let start_time = Instant::now();

        let mem = &mut self.mem;
        let result = MemOps::with_raw(iter, out, out_fail, |data| mem.phys_write_raw_iter(data));

        self.writes
            .add(start_time.elapsed().as_secs_f64(), number_of_bytes);

        //if self.writes.total_count() % 10000 == 0 {
        if self.last_write_info.elapsed().as_secs_f64() >= 1f64 {
            info!(
                "Write Metrics: writes_per_second={} average_latency={:.4}ms; average_bytes={}; bytes_per_second={}",
                self.writes.len(),
                self.writes.average_latency().unwrap_or_default() * 1000f64,
                self.writes.average_bytes().unwrap_or_default(),
                self.writes.bandwidth().unwrap_or_default(),
            );
            self.last_write_info = Instant::now();
        }

        result
    }

    #[inline]
    fn metadata(&self) -> PhysicalMemoryMetadata {
        self.mem.metadata()
    }

    #[inline]
    fn set_mem_map(&mut self, mem_map: &[PhysicalMemoryMapping]) {
        self.mem.set_mem_map(mem_map)
    }
}

#[cfg(feature = "plugins")]
::cglue::cglue_impl_group!(
    PhysicalMemoryMetrics<T: PhysicalMemory>,
    crate::plugins::ConnectorInstance,
    {}
);

/// This struct tracks latency and length of recent read and write operations.
///
/// It has a minimum and maximum length, as well as a maximum storage time.
/// * The minimum length is to ensure you have enough data for an estimate.
/// * The maximum length is to make sure the history doesn't take up too much space.
/// * The maximum age is to make sure the estimate isn't outdated.
///
/// Time difference between values can be zero, but never negative.
///
/// This implementation is derived from (egui)[https://github.com/emilk/egui/blob/1c8cf9e3d59d8aee4c073b9e17695ee85c40bdbf/crates/emath/src/history.rs].
#[derive(Clone, Debug)]
struct MemOpsHistory {
    start_time: Instant,

    /// In elements, i.e. of `values.len()`.
    /// The length is initially zero, but once past `min_len` will not shrink below it.
    min_len: usize,

    /// In elements, i.e. of `values.len()`.
    max_len: usize,

    /// In seconds.
    max_age: f32,

    /// Total number of elements seen ever
    total_count: u64,

    /// (time, value) pairs, oldest front, newest back.
    /// Time difference between values can be zero, but never negative.
    values: VecDeque<(f64, MemOpsHistoryEntry)>,
}

#[derive(Clone, Copy, Debug)]
struct MemOpsHistoryEntry {
    pub latency: f64, // secs
    pub bytes: usize, // bytes
}

#[allow(unused)]
impl MemOpsHistory {
    pub fn new(length_range: std::ops::Range<usize>, max_age: f32) -> Self {
        Self {
            start_time: Instant::now(),
            min_len: length_range.start,
            max_len: length_range.end,
            max_age,
            total_count: 0,
            values: Default::default(),
        }
    }

    #[inline]
    pub fn max_len(&self) -> usize {
        self.max_len
    }

    #[inline]
    pub fn max_age(&self) -> f32 {
        self.max_age
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    /// Current number of values kept in history
    #[inline]
    pub fn len(&self) -> usize {
        self.values.len()
    }

    /// Total number of values seen.
    /// Includes those that have been discarded due to `max_len` or `max_age`.
    #[inline]
    pub fn total_count(&self) -> u64 {
        self.total_count
    }

    #[inline]
    pub fn clear(&mut self) {
        self.values.clear();
    }

    /// Values must be added with a monotonically increasing time, or at least not decreasing.
    pub fn add(&mut self, latency: f64, bytes: usize) {
        let now = self.start_time.elapsed().as_secs_f64();
        if let Some((last_time, _)) = self.values.back() {
            assert!(now >= *last_time, "Time shouldn't move backwards");
        }
        self.total_count += 1;
        self.values
            .push_back((now, MemOpsHistoryEntry { latency, bytes }));
        self.flush();
    }

    /// Mean time difference between values in this [`History`].
    pub fn mean_time_interval(&self) -> Option<f64> {
        if let (Some(first), Some(last)) = (self.values.front(), self.values.back()) {
            let n = self.len();
            if n >= 2 {
                Some((last.0 - first.0) / ((n - 1) as f64))
            } else {
                None
            }
        } else {
            None
        }
    }

    // Mean number of events per second.
    pub fn rate(&self) -> Option<f64> {
        self.mean_time_interval().map(|time| 1.0 / time)
    }

    /// Remove samples that are too old.
    pub fn flush(&mut self) {
        let now = self.start_time.elapsed().as_secs_f64();
        while self.values.len() > self.max_len {
            self.values.pop_front();
        }
        while self.values.len() > self.min_len {
            if let Some((front_time, _)) = self.values.front() {
                if *front_time < now - (self.max_age as f64) {
                    self.values.pop_front();
                } else {
                    break;
                }
            } else {
                break;
            }
        }
    }

    /// Returns the sum of all latencys
    #[inline]
    pub fn sum_latency(&self) -> f64 {
        self.values.iter().map(|(_, value)| value.latency).sum()
    }

    /// Returns the average latency
    pub fn average_latency(&self) -> Option<f64> {
        let num = self.len();
        if num > 0 {
            Some(self.sum_latency() / (num as f64))
        } else {
            None
        }
    }

    /// Returns the sum of bytes transmitted
    #[inline]
    pub fn sum_bytes(&self) -> usize {
        self.values.iter().map(|(_, value)| value.bytes).sum()
    }

    /// Returns the average number of bytes transmitted
    pub fn average_bytes(&self) -> Option<usize> {
        let num = self.len();
        if num > 0 {
            Some((self.sum_bytes() as f64 / (num as f64)) as usize)
        } else {
            None
        }
    }

    /// Returns the number of bytes per second
    pub fn bandwidth(&self) -> Option<usize> {
        Some((self.average_bytes()? as f64 * self.rate()?) as usize)
    }
}
