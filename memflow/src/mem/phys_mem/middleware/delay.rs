use ::std::{thread, time::Duration};

use crate::error::Result;
use crate::mem::{
    PhysicalMemory, PhysicalMemoryMapping, PhysicalMemoryMetadata, PhysicalReadMemOps,
    PhysicalWriteMemOps,
};

/// The cache object that can use as a drop-in replacement for any Connector.
///
/// Since this cache implements [`PhysicalMemory`] it can be used as a replacement
/// in all structs and functions that require a [`PhysicalMemory`] object.
pub struct DelayedPhysicalMemory<T> {
    mem: T,
    // TODO: jitter, etc.
    delay: Duration,
}

impl<T> Clone for DelayedPhysicalMemory<T>
where
    T: Clone,
{
    fn clone(&self) -> Self {
        Self {
            mem: self.mem.clone(),
            delay: self.delay,
        }
    }
}

impl<T: PhysicalMemory> DelayedPhysicalMemory<T> {
    /// Constructs a new cache based on the given `PageCache`.
    ///
    /// This function is used when manually constructing a cache inside of the memflow crate itself.
    ///
    /// For general usage it is advised to just use the [builder](struct.DelayedPhysicalMemoryBuilder.html)
    /// to construct the cache.
    pub fn new(mem: T, delay: Duration) -> Self {
        Self { mem, delay }
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
    /// use memflow::mem::{PhysicalMemory, DelayedPhysicalMemory, MemoryView};
    ///
    /// fn build<T: PhysicalMemory>(mem: T) -> T {
    ///     let mut cache = DelayedPhysicalMemory::builder(mem)
    ///         .arch(x64::ARCH)
    ///         .build()
    ///         .unwrap();
    ///
    ///     // use the cache...
    ///     let value: u64 = cache.phys_view().read(0.into()).unwrap();
    ///     assert_eq!(value, MAGIC_VALUE);
    ///
    ///     // retrieve ownership of mem and return it back
    ///     cache.into_inner()
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

impl<'a, T: PhysicalMemory> DelayedPhysicalMemory<T> {
    /// Returns a new builder for this cache with default settings.
    pub fn builder(mem: T) -> DelayedPhysicalMemoryBuilder<T> {
        DelayedPhysicalMemoryBuilder::new(mem)
    }
}

// forward PhysicalMemory trait fncs
impl<T: PhysicalMemory> PhysicalMemory for DelayedPhysicalMemory<T> {
    fn phys_read_raw_iter(
        &mut self,
        //data: PhysicalReadMemOps,
        data: PhysicalReadMemOps,
    ) -> Result<()> {
        thread::sleep(self.delay);
        self.mem.phys_read_raw_iter(data)
    }

    fn phys_write_raw_iter(&mut self, data: PhysicalWriteMemOps) -> Result<()> {
        thread::sleep(self.delay);
        self.mem.phys_write_raw_iter(data)
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

/// The builder interface for constructing a `DelayedPhysicalMemory` object.
pub struct DelayedPhysicalMemoryBuilder<T> {
    mem: T,
    delay: Duration,
}

impl<T: PhysicalMemory> DelayedPhysicalMemoryBuilder<T> {
    /// Creates a new `DelayedPhysicalMemory` builder.
    /// The memory object is mandatory as the DelayedPhysicalMemory struct wraps around it.
    ///
    /// This type of cache also is required to know the exact page size of the target system.
    /// This can either be set directly via the `page_size()` method or via the `arch()` method.
    /// If no page size has been set this builder will fail to build the DelayedPhysicalMemory.
    ///
    /// Without further adjustments this function creates a cache that is 2 megabytes in size and caches
    /// pages that contain pagetable entries as well as read-only pages.
    ///
    /// It is also possible to either let the `DelayedPhysicalMemory` object own or just borrow the underlying memory object.
    ///
    /// # Examples
    /// Moves ownership of a mem object and retrieves it back:
    /// ```
    /// # const MAGIC_VALUE: u64 = 0x23bd_318f_f3a3_5821;
    /// use memflow::architecture::x86::x64;
    /// use memflow::mem::{PhysicalMemory, DelayedPhysicalMemory, MemoryView};
    ///
    /// fn build<T: PhysicalMemory>(mem: T) {
    ///     let mut cache = DelayedPhysicalMemory::builder(mem)
    ///         .arch(x64::ARCH)
    ///         .build()
    ///         .unwrap();
    ///
    ///     cache.phys_write(0.into(), &MAGIC_VALUE);
    ///
    ///     let mut mem = cache.into_inner();
    ///
    ///     let value: u64 = mem.phys_view().read(0.into()).unwrap();
    ///     assert_eq!(value, MAGIC_VALUE);
    /// }
    /// # use memflow::dummy::DummyMemory;
    /// # use memflow::types::size;
    /// # let mut mem = DummyMemory::new(size::mb(4));
    /// # mem.phys_write(0.into(), &0xffaaffaau64).unwrap();
    /// # build(mem);
    /// ```
    ///
    /// Borrowing a mem object:
    /// ```
    /// # const MAGIC_VALUE: u64 = 0x23bd_318f_f3a3_5821;
    /// use memflow::architecture::x86::x64;
    /// use memflow::mem::{PhysicalMemory, DelayedPhysicalMemory, MemoryView};
    /// use memflow::cglue::{Fwd, ForwardMut};
    ///
    /// fn build<T: PhysicalMemory>(mem: Fwd<&mut T>)
    ///     -> impl PhysicalMemory + '_ {
    ///     DelayedPhysicalMemory::builder(mem)
    ///         .arch(x64::ARCH)
    ///         .build()
    ///         .unwrap()
    /// }
    ///
    /// # use memflow::dummy::DummyMemory;
    /// # use memflow::types::size;
    /// # let mut mem = DummyMemory::new(size::mb(4));
    /// # mem.phys_write(0.into(), &MAGIC_VALUE).unwrap();
    /// let mut cache = build(mem.forward_mut());
    ///
    /// let value: u64 = cache.phys_view().read(0.into()).unwrap();
    /// assert_eq!(value, MAGIC_VALUE);
    ///
    /// cache.phys_write(0.into(), &0u64).unwrap();
    ///
    /// // We drop the cache and are able to use mem again
    /// std::mem::drop(cache);
    ///
    /// let value: u64 = mem.phys_view().read(0.into()).unwrap();
    /// assert_ne!(value, MAGIC_VALUE);
    /// ```
    pub fn new(mem: T) -> Self {
        Self {
            mem,
            delay: Duration::from_millis(10),
        }
    }

    /// Changes the page size of the cache.
    ///
    /// The cache has to know the exact page size of the target system internally to give reasonable performance.
    /// The page size can be either set directly via this function or it can be fetched from the `Architecture`
    /// via the `arch()` method of the builder.
    ///
    /// If the page size is not set the builder will fail.
    ///
    /// # Examples
    ///
    /// ```
    /// use memflow::types::size;
    /// use memflow::mem::{PhysicalMemory, DelayedPhysicalMemory};
    ///
    /// fn build<T: PhysicalMemory>(mem: T) {
    ///     let cache = DelayedPhysicalMemory::builder(mem)
    ///         .page_size(size::kb(4))
    ///         .build()
    ///         .unwrap();
    /// }
    /// # use memflow::dummy::DummyMemory;
    /// # let mut mem = DummyMemory::new(size::mb(4));
    /// # build(mem);
    /// ```
    pub fn delay(mut self, delay: Duration) -> Self {
        self.delay = delay;
        self
    }

    /// Builds the `DelayedPhysicalMemory` object or returns an error.
    pub fn build<'a>(self) -> Result<DelayedPhysicalMemory<T>> {
        Ok(DelayedPhysicalMemory::new(self.mem, self.delay))
    }
}

#[cfg(feature = "plugins")]
cglue::cglue_impl_group!(
    DelayedPhysicalMemory<T: PhysicalMemory>,
    crate::plugins::ConnectorInstance,
    {}
);
