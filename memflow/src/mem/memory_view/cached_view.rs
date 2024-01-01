//! Caching layer for a memory view.
use super::*;
use crate::mem::phys_mem::{page_cache::PageCache, PhysicalMemoryView};

/// Cached memory view.
///
/// This structure allows to build a page cache on top of a memory view.
///
/// Internally this structure uses the [`CachedPhysicalMemory`] cache.
/// It does this by remapping from / to [`PhysicalMemory`].
#[derive(Clone)]
pub struct CachedView<'a, T, Q>
where
    T: MemoryView,
    Q: CacheValidator,
{
    mem: PhysicalMemoryView<CachedPhysicalMemory<'a, PhysicalMemoryOnView<T>, Q>>,
}

impl<'a, T, Q> MemoryView for CachedView<'a, T, Q>
where
    T: MemoryView,
    Q: CacheValidator,
{
    #[inline]
    fn read_raw_iter(&mut self, data: ReadRawMemOps) -> Result<()> {
        self.mem.read_raw_iter(data)
    }

    #[inline]
    fn write_raw_iter(&mut self, data: WriteRawMemOps) -> Result<()> {
        self.mem.write_raw_iter(data)
    }

    #[inline]
    fn metadata(&self) -> MemoryViewMetadata {
        self.mem.metadata()
    }
}

impl<'a, T: MemoryView> CachedView<'a, T, DefaultCacheValidator> {
    /// Returns a new builder for this cache with default settings.
    #[inline]
    pub fn builder(mem: T) -> CachedViewBuilder<T, DefaultCacheValidator> {
        CachedViewBuilder::new(mem)
    }
}

pub struct CachedViewBuilder<T, Q> {
    mem: T,
    validator: Q,
    page_size: Option<usize>,
    cache_size: usize,
    page_type_mask: PageType,
}

impl<T: MemoryView> CachedViewBuilder<T, DefaultCacheValidator> {
    /// Creates a new [`CachedView`] builder.
    /// The memory object is mandatory as the [`CachedView`] struct wraps around it.
    ///
    /// This type of cache also is required to know the exact page size of the target system.
    /// This can either be set directly via the `page_size()` method or via the `arch()` method.
    /// If no page size has been set this builder will fail to build the [`CachedView`].
    ///
    /// Without further adjustments this function creates a cache that is 2 megabytes in size and caches
    /// pages that contain pagetable entries as well as read-only pages.
    ///
    /// It is also possible to either let the [`CachedView`] object own or just borrow the underlying memory object.
    ///
    /// # Examples
    /// Moves ownership of a mem object and retrieves it back:
    /// ```
    /// # const MAGIC_VALUE: u64 = 0x23bd_318f_f3a3_5821;
    /// use memflow::architecture::x86::x64;
    /// use memflow::mem::{PhysicalMemory, CachedPhysicalMemory, MemoryView};
    ///
    /// fn build<T: PhysicalMemory>(mem: T) {
    ///     let mut cache = CachedPhysicalMemory::builder(mem)
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
    /// use memflow::mem::{PhysicalMemory, CachedPhysicalMemory, MemoryView};
    /// use memflow::cglue::{Fwd, ForwardMut};
    ///
    /// fn build<T: PhysicalMemory>(mem: Fwd<&mut T>)
    ///     -> impl PhysicalMemory + '_ {
    ///     CachedPhysicalMemory::builder(mem)
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
            validator: DefaultCacheValidator::default(),
            page_size: None,
            cache_size: size::mb(2),
            page_type_mask: PageType::PAGE_TABLE | PageType::READ_ONLY,
        }
    }
}

impl<T: MemoryView, Q: CacheValidator> CachedViewBuilder<T, Q> {
    /// Builds the [`CachedView`] object or returns an error if the page size is not set.
    pub fn build<'a>(self) -> Result<CachedView<'a, T, Q>> {
        let phys_mem = self.mem.into_phys_mem();

        let cache = CachedPhysicalMemory::new(
            phys_mem,
            PageCache::with_page_size(
                self.page_size.ok_or_else(|| {
                    Error(ErrorOrigin::Cache, ErrorKind::Uninitialized)
                        .log_error("page_size must be initialized")
                })?,
                self.cache_size,
                self.page_type_mask,
                self.validator,
            ),
        );

        Ok(CachedView {
            mem: cache.into_mem_view(),
        })
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
    /// use memflow::mem::{PhysicalMemory, CachedPhysicalMemory};
    /// use memflow::types::DefaultCacheValidator;
    ///
    /// fn build<T: PhysicalMemory>(mem: T) {
    ///     let cache = CachedPhysicalMemory::builder(mem)
    ///         .arch(x64::ARCH)
    ///         .validator(DefaultCacheValidator::new(Duration::from_millis(2000).into()))
    ///         .build()
    ///         .unwrap();
    /// }
    /// # use memflow::dummy::DummyMemory;
    /// # use memflow::types::size;
    /// # let mut mem = DummyMemory::new(size::mb(4));
    /// # build(mem);
    /// ```
    pub fn validator<QN: CacheValidator>(self, validator: QN) -> CachedViewBuilder<T, QN> {
        CachedViewBuilder {
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
    /// use memflow::mem::{PhysicalMemory, CachedPhysicalMemory};
    ///
    /// fn build<T: PhysicalMemory>(mem: T) {
    ///     let cache = CachedPhysicalMemory::builder(mem)
    ///         .page_size(size::kb(4))
    ///         .build()
    ///         .unwrap();
    /// }
    /// # use memflow::dummy::DummyMemory;
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
    /// use memflow::mem::{PhysicalMemory, CachedPhysicalMemory};
    ///
    /// fn build<T: PhysicalMemory>(mem: T) {
    ///     let cache = CachedPhysicalMemory::builder(mem)
    ///         .arch(x64::ARCH)
    ///         .build()
    ///         .unwrap();
    /// }
    /// # use memflow::dummy::DummyMemory;
    /// # use memflow::types::size;
    /// # let mut mem = DummyMemory::new(size::mb(4));
    /// # build(mem);
    /// ```
    pub fn arch(mut self, arch: impl Into<ArchitectureObj>) -> Self {
        self.page_size = Some(arch.into().page_size());
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
    /// use memflow::mem::{PhysicalMemory, CachedPhysicalMemory};
    ///
    /// fn build<T: PhysicalMemory>(mem: T) {
    ///     let cache = CachedPhysicalMemory::builder(mem)
    ///         .arch(x64::ARCH)
    ///         .cache_size(size::mb(2))
    ///         .build()
    ///         .unwrap();
    /// }
    /// # use memflow::dummy::DummyMemory;
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
    /// use memflow::mem::{PhysicalMemory, CachedPhysicalMemory};
    ///
    /// fn build<T: PhysicalMemory>(mem: T) {
    ///     let cache = CachedPhysicalMemory::builder(mem)
    ///         .arch(x32::ARCH)
    ///         .page_type_mask(PageType::PAGE_TABLE | PageType::READ_ONLY)
    ///         .build()
    ///         .unwrap();
    /// }
    /// # use memflow::dummy::DummyMemory;
    /// # use memflow::types::size;
    /// # let mut mem = DummyMemory::new(size::mb(4));
    /// # build(mem);
    /// ```
    pub fn page_type_mask(mut self, page_type_mask: PageType) -> Self {
        self.page_type_mask = page_type_mask;
        self
    }
}
