/*!
This cache is a wrapper for connector objects that implement the `PhysicalMemory` trait.
It enables a configurable caching layer when accessing physical pages.

Each page that is being read by the the connector will be placed into a `PageCache` object.
If the cache is still valid then for consecutive reads this connector will just return the values from the cache
and not issue out a new read. In case the cache is not valid anymore it will do a new read.

The cache time is determined by the customizable cache validator.
The cache validator has to implement the [`CacheValidator`](../trait.CacheValidator.html) trait.

To make it easier and quicker to construct and work with caches this module also contains a cache builder.

More examples can be found in the documentations for each of the structs in this module.

# Examples

Building a simple cache with default settings:
```
use memflow::architecture::x86::x64;
use memflow::mem::{PhysicalMemory, CachedMemoryAccess};

fn build<T: PhysicalMemory>(mem: T) {
    let cache = CachedMemoryAccess::builder(mem)
        .arch(x64::ARCH)
        .build()
        .unwrap();
}
```
*/

use super::{
    page_cache::PageCache, page_cache::PageValidity, CacheValidator, DefaultCacheValidator,
};
use crate::architecture::ArchitectureObj;
use crate::error::Result;
use crate::iter::PageChunks;
use crate::mem::phys_mem::{
    PhysicalMemory, PhysicalMemoryMetadata, PhysicalReadData, PhysicalWriteData,
};
use crate::types::{size, PageType};

use bumpalo::Bump;

/// The cache object that can use as a drop-in replacement for any Connector.
///
/// Since this cache implements `PhysicalMemory` it can be used as a replacement
/// in all structs and functions that require a `PhysicalMemory` object.
pub struct CachedMemoryAccess<'a, T, Q> {
    mem: T,
    cache: PageCache<'a, Q>,
    arena: Bump,
}

impl<'a, T, Q> Clone for CachedMemoryAccess<'a, T, Q>
where
    T: Clone,
    Q: CacheValidator + Clone,
{
    fn clone(&self) -> Self {
        Self {
            mem: self.mem.clone(),
            cache: self.cache.clone(),
            arena: Bump::new(),
        }
    }
}

impl<'a, T: PhysicalMemory, Q: CacheValidator> CachedMemoryAccess<'a, T, Q> {
    /// Constructs a new cache based on the given `PageCache`.
    ///
    /// This function is used when manually constructing a cache inside of the memflow crate itself.
    ///
    /// For general usage it is advised to just use the [builder](struct.CachedMemoryAccessBuilder.html)
    /// to construct the cache.
    pub fn new(mem: T, cache: PageCache<'a, Q>) -> Self {
        Self {
            mem,
            cache,
            arena: Bump::new(),
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
    /// use memflow::mem::{PhysicalMemory, CachedMemoryAccess};
    ///
    /// fn build<T: PhysicalMemory>(mem: T) -> T {
    ///     let mut cache = CachedMemoryAccess::builder(mem)
    ///         .arch(x64::ARCH)
    ///         .build()
    ///         .unwrap();
    ///
    ///     // use the cache...
    ///     let value: u64 = cache.phys_read(0.into()).unwrap();
    ///     assert_eq!(value, MAGIC_VALUE);
    ///
    ///     // retrieve ownership of mem and return it back
    ///     cache.destroy()
    /// }
    /// # use memflow::mem::dummy::DummyMemory;
    /// # use memflow::types::size;
    /// # let mut mem = DummyMemory::new(size::mb(4));
    /// # mem.phys_write(0.into(), &MAGIC_VALUE).unwrap();
    /// # build(mem);
    /// ```
    pub fn destroy(self) -> T {
        self.mem
    }
}

impl<'a, T: PhysicalMemory> CachedMemoryAccess<'a, T, DefaultCacheValidator> {
    /// Returns a new builder for this cache with default settings.
    pub fn builder(mem: T) -> CachedMemoryAccessBuilder<T, DefaultCacheValidator> {
        CachedMemoryAccessBuilder::new(mem)
    }
}

// forward PhysicalMemory trait fncs
impl<'a, T: PhysicalMemory, Q: CacheValidator> PhysicalMemory for CachedMemoryAccess<'a, T, Q> {
    fn phys_read_raw_list(&mut self, data: &mut [PhysicalReadData]) -> Result<()> {
        self.cache.validator.update_validity();
        self.arena.reset();
        self.cache.cached_read(&mut self.mem, data, &self.arena)
    }

    fn phys_write_raw_list(&mut self, data: &[PhysicalWriteData]) -> Result<()> {
        self.cache.validator.update_validity();

        let cache = &mut self.cache;
        let mem = &mut self.mem;

        data.iter().for_each(move |PhysicalWriteData(addr, data)| {
            if cache.is_cached_page_type(addr.page_type()) {
                for (paddr, data_chunk) in data.page_chunks(addr.address(), cache.page_size()) {
                    let mut cached_page = cache.cached_page_mut(paddr, false);
                    if let PageValidity::Valid(buf) = &mut cached_page.validity {
                        // write-back into still valid cache pages
                        let start = paddr - cached_page.address;
                        buf[start..(start + data_chunk.len())].copy_from_slice(data_chunk);
                    }

                    cache.put_entry(cached_page);
                }
            }
        });

        mem.phys_write_raw_list(data)
    }

    fn metadata(&self) -> PhysicalMemoryMetadata {
        self.mem.metadata()
    }
}

/// The builder interface for constructing a `CachedMemoryAccess` object.
pub struct CachedMemoryAccessBuilder<T, Q> {
    mem: T,
    validator: Q,
    page_size: Option<usize>,
    cache_size: usize,
    page_type_mask: PageType,
}

impl<T: PhysicalMemory> CachedMemoryAccessBuilder<T, DefaultCacheValidator> {
    /// Creates a new `CachedMemoryAccess` builder.
    /// The memory object is mandatory as the CachedMemoryAccess struct wraps around it.
    ///
    /// This type of cache also is required to know the exact page size of the target system.
    /// This can either be set directly via the `page_size()` method or via the `arch()` method.
    /// If no page size has been set this builder will fail to build the CachedMemoryAccess.
    ///
    /// Without further adjustments this function creates a cache that is 2 megabytes in size and caches
    /// pages that contain pagetable entries as well as read-only pages.
    ///
    /// It is also possible to either let the `CachedMemoryAccess` object own or just borrow the underlying memory object.
    ///
    /// # Examples
    /// Moves ownership of a mem object and retrieves it back:
    /// ```
    /// # const MAGIC_VALUE: u64 = 0x23bd_318f_f3a3_5821;
    /// use memflow::architecture::x86::x64;
    /// use memflow::mem::{PhysicalMemory, CachedMemoryAccess};
    ///
    /// fn build<T: PhysicalMemory>(mem: T) {
    ///     let mut cache = CachedMemoryAccess::builder(mem)
    ///         .arch(x64::ARCH)
    ///         .build()
    ///         .unwrap();
    ///
    ///     cache.phys_write(0.into(), &MAGIC_VALUE);
    ///
    ///     let mut mem = cache.destroy();
    ///
    ///     let value: u64 = mem.phys_read(0.into()).unwrap();
    ///     assert_eq!(value, MAGIC_VALUE);
    /// }
    /// # use memflow::mem::dummy::DummyMemory;
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
    /// use memflow::mem::{PhysicalMemory, CachedMemoryAccess};
    ///
    /// fn build<T: PhysicalMemory>(mem: &mut T)
    ///     -> impl PhysicalMemory + '_ {
    ///     CachedMemoryAccess::builder(mem)
    ///         .arch(x64::ARCH)
    ///         .build()
    ///         .unwrap()
    /// }
    ///
    /// # use memflow::mem::dummy::DummyMemory;
    /// # use memflow::types::size;
    /// # let mut mem = DummyMemory::new(size::mb(4));
    /// # mem.phys_write(0.into(), &MAGIC_VALUE).unwrap();
    /// let mut cache = build(&mut mem);
    ///
    /// let value: u64 = cache.phys_read(0.into()).unwrap();
    /// assert_eq!(value, MAGIC_VALUE);
    ///
    /// cache.phys_write(0.into(), &0u64).unwrap();
    ///
    /// // We drop the cache and are able to use mem again
    /// std::mem::drop(cache);
    ///
    /// let value: u64 = mem.phys_read(0.into()).unwrap();
    /// assert_ne!(value, MAGIC_VALUE);
    /// ```
    pub fn new(mem: T) -> Self {
        Self {
            mem,
            validator: DefaultCacheValidator::default(),
            page_size: None,
            cache_size: size::mb(2),
            page_type_mask: PageType::PAGE_TABLE | PageType::READ_ONLY,
        }
    }
}

impl<T: PhysicalMemory, Q: CacheValidator> CachedMemoryAccessBuilder<T, Q> {
    /// Builds the `CachedMemoryAccess` object or returns an error if the page size is not set.
    pub fn build<'a>(self) -> Result<CachedMemoryAccess<'a, T, Q>> {
        Ok(CachedMemoryAccess::new(
            self.mem,
            PageCache::with_page_size(
                self.page_size.ok_or("page_size must be initialized")?,
                self.cache_size,
                self.page_type_mask,
                self.validator,
            ),
        ))
    }

    /// Sets a custom validator for the cache.
    ///
    /// If this function is not called it will default to a [`DefaultCacheValidator`](../timed_validator/index.html)
    /// for std builds and a /* TODO */ validator for no_std builds.
    ///
    /// The default setting is `DefaultCacheValidator::default()`.
    ///
    /// # Examples:
    ///
    /// ```
    /// use std::time::Duration;
    ///
    /// use memflow::architecture::x86::x64;
    /// use memflow::mem::{PhysicalMemory, CachedMemoryAccess, DefaultCacheValidator};
    ///
    /// fn build<T: PhysicalMemory>(mem: T) {
    ///     let cache = CachedMemoryAccess::builder(mem)
    ///         .arch(x64::ARCH)
    ///         .validator(DefaultCacheValidator::new(Duration::from_millis(2000).into()))
    ///         .build()
    ///         .unwrap();
    /// }
    /// # use memflow::mem::dummy::DummyMemory;
    /// # use memflow::types::size;
    /// # let mut mem = DummyMemory::new(size::mb(4));
    /// # build(mem);
    /// ```
    pub fn validator<QN: CacheValidator>(self, validator: QN) -> CachedMemoryAccessBuilder<T, QN> {
        CachedMemoryAccessBuilder {
            mem: self.mem,
            validator,
            page_size: self.page_size,
            cache_size: self.cache_size,
            page_type_mask: self.page_type_mask,
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
    /// use memflow::mem::{PhysicalMemory, CachedMemoryAccess};
    ///
    /// fn build<T: PhysicalMemory>(mem: T) {
    ///     let cache = CachedMemoryAccess::builder(mem)
    ///         .page_size(size::kb(4))
    ///         .build()
    ///         .unwrap();
    /// }
    /// # use memflow::mem::dummy::DummyMemory;
    /// # let mut mem = DummyMemory::new(size::mb(4));
    /// # build(mem);
    /// ```
    pub fn page_size(mut self, page_size: usize) -> Self {
        self.page_size = Some(page_size);
        self
    }

    /// Retrieves the page size for this cache from the given `Architecture`.
    ///
    /// The cache has to know the exact page size of the target system internally to give reasonable performance.
    /// The page size can be either fetched from the `Architecture` via this method or it can be set directly
    /// via the `page_size()` method of the builder.
    ///
    /// If the page size is not set the builder will fail.
    ///
    /// # Examples
    ///
    /// ```
    /// use memflow::architecture::x86::x64;
    /// use memflow::mem::{PhysicalMemory, CachedMemoryAccess};
    ///
    /// fn build<T: PhysicalMemory>(mem: T) {
    ///     let cache = CachedMemoryAccess::builder(mem)
    ///         .arch(x64::ARCH)
    ///         .build()
    ///         .unwrap();
    /// }
    /// # use memflow::mem::dummy::DummyMemory;
    /// # use memflow::types::size;
    /// # let mut mem = DummyMemory::new(size::mb(4));
    /// # build(mem);
    /// ```
    pub fn arch(mut self, arch: ArchitectureObj) -> Self {
        self.page_size = Some(arch.page_size());
        self
    }

    /// Sets the total amount of cache to be used.
    ///
    /// This is the total amount of cache (in bytes) this page cache will allocate.
    /// Ideally you'd want to keep this value low enough so that most of the cache stays in the lower level caches of your cpu.
    ///
    /// The default setting is 2 megabytes.
    ///
    /// This setting can drastically impact the performance of the cache.
    ///
    /// # Examples:
    ///
    /// ```
    /// use memflow::types::size;
    /// use memflow::architecture::x86::x64;
    /// use memflow::mem::{PhysicalMemory, CachedMemoryAccess};
    ///
    /// fn build<T: PhysicalMemory>(mem: T) {
    ///     let cache = CachedMemoryAccess::builder(mem)
    ///         .arch(x64::ARCH)
    ///         .cache_size(size::mb(2))
    ///         .build()
    ///         .unwrap();
    /// }
    /// # use memflow::mem::dummy::DummyMemory;
    /// # let mut mem = DummyMemory::new(size::mb(4));
    /// # build(mem);
    /// ```
    pub fn cache_size(mut self, cache_size: usize) -> Self {
        self.cache_size = cache_size;
        self
    }

    /// Adjusts the type of pages that the cache will hold in it's cache.
    ///
    /// The page type can be a bitmask that contains one or multiple page types.
    /// All page types matching this bitmask will be kept in the cache.
    /// All pages that are not matching the bitmask will be re-read/re-written on every request.
    ///
    /// The default setting is `PageType::PAGE_TABLE | PageType::READ_ONLY`.
    ///
    /// This setting can drastically impact the performance of the cache.
    ///
    /// # Examples:
    ///
    /// ```
    /// use memflow::types::PageType;
    /// use memflow::architecture::x86::x32;
    /// use memflow::mem::{PhysicalMemory, CachedMemoryAccess};
    ///
    /// fn build<T: PhysicalMemory>(mem: T) {
    ///     let cache = CachedMemoryAccess::builder(mem)
    ///         .arch(x32::ARCH)
    ///         .page_type_mask(PageType::PAGE_TABLE | PageType::READ_ONLY)
    ///         .build()
    ///         .unwrap();
    /// }
    /// # use memflow::mem::dummy::DummyMemory;
    /// # use memflow::types::size;
    /// # let mut mem = DummyMemory::new(size::mb(4));
    /// # build(mem);
    /// ```
    pub fn page_type_mask(mut self, page_type_mask: PageType) -> Self {
        self.page_type_mask = page_type_mask;
        self
    }
}
