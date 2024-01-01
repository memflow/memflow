//! This cache is a wrapper for connector objects that implement the [`PhysicalMemory`] trait.
//! It enables a configurable caching layer when accessing physical pages.
//!
//! Each page that is being read by the the connector will be placed into a `PageCache` object.
//! If the cache is still valid then for consecutive reads this connector will just return the values from the cache
//! and not issue out a new read. In case the cache is not valid anymore it will do a new read.
//!
//! The cache time is determined by the customizable cache validator.
//! The cache validator has to implement the [`CacheValidator`](../trait.CacheValidator.html) trait.
//!
//! To make it easier and quicker to construct and work with caches this module also contains a cache builder.
//!
//! More examples can be found in the documentations for each of the structs in this module.
//!
//! # Examples
//!
//! Building a simple cache with default settings:
//! ```
//! # const MAGIC_VALUE: u64 = 0x23bd_318f_f3a3_5821;
//! use memflow::prelude::v1::*;
//! use memflow::dummy::DummyMemory;
//! # use memflow::dummy::DummyOs;
//! # use memflow::architecture::x86::x64;
//!
//! # let phys_mem = DummyMemory::new(size::mb(16));
//! # let mut os = DummyOs::new(phys_mem);
//! # let (dtb, virt_base) = os.alloc_dtb(size::mb(8), &[]);
//! # let phys_mem = os.into_inner();
//! # let translator = x64::new_translator(dtb);
//! let mut virt_mem = VirtualDma::new(phys_mem, x64::ARCH, translator);
//!
//! let mut cached_mem = CachedView::builder(virt_mem)
//!     .arch(x64::ARCH)
//!     .validator(DefaultCacheValidator::default())
//!     .cache_size(size::mb(1))
//!     .build()
//!     .unwrap();
//!
//! let addr = virt_base; // some arbitrary address
//!
//! cached_mem.write(addr, &MAGIC_VALUE).unwrap();
//!
//! let value: u64 = cached_mem.read(addr).unwrap();
//! assert_eq!(value, MAGIC_VALUE);
//! ```

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
    /// use memflow::prelude::v1::*;
    /// use memflow::dummy::DummyMemory;
    /// # use memflow::dummy::DummyOs;
    /// # use memflow::architecture::x86::x64;
    ///
    /// # let phys_mem = DummyMemory::new(size::mb(16));
    /// # let mut os = DummyOs::new(phys_mem);
    /// # let (dtb, virt_base) = os.alloc_dtb(size::mb(8), &[]);
    /// # let phys_mem = os.into_inner();
    /// # let translator = x64::new_translator(dtb);
    /// let mut virt_mem = VirtualDma::new(phys_mem, x64::ARCH, translator);
    ///
    /// let mut cached_mem = CachedView::builder(virt_mem)
    ///     .arch(x64::ARCH)
    ///     .build()
    ///     .unwrap();
    ///
    /// let addr = virt_base; // some arbitrary address
    ///
    /// cached_mem.write(addr, &MAGIC_VALUE).unwrap();
    ///
    /// let value: u64 = cached_mem.read(addr).unwrap();
    /// assert_eq!(value, MAGIC_VALUE);
    /// ```
    ///
    /// Borrowing a mem object:
    /// ```
    /// use memflow::prelude::v1::*;
    /// use memflow::dummy::DummyMemory;
    /// # use memflow::dummy::DummyOs;
    /// # use memflow::architecture::x86::x64;
    ///
    /// fn build<T: MemoryView>(mem: Fwd<&mut T>)
    ///     -> impl MemoryView + '_ {
    ///     CachedView::builder(mem)
    ///         .arch(x64::ARCH)
    ///         .build()
    ///         .unwrap()
    /// }
    ///
    /// # let phys_mem = DummyMemory::new(size::mb(16));
    /// # let mut os = DummyOs::new(phys_mem);
    /// # let (dtb, virt_base) = os.alloc_dtb(size::mb(8), &[]);
    /// # let phys_mem = os.into_inner();
    /// # let translator = x64::new_translator(dtb);
    /// let mut virt_mem = VirtualDma::new(phys_mem, x64::ARCH, translator);
    /// let mut cached_view = build(virt_mem.forward_mut());
    ///
    /// let read = cached_view.read::<u32>(0.into()).unwrap();
    /// ```
    pub fn new(mem: T) -> Self {
        Self {
            mem,
            validator: DefaultCacheValidator::default(),
            page_size: None,
            cache_size: size::mb(2),
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
                // we do not know pagetypes on virtual memory so we have to apply this cache to all types
                PageType::all(),
                self.validator,
            ),
        );

        Ok(CachedView {
            mem: cache.into_mem_view(),
        })
    }

    /// Sets a custom validator for the cache.
    ///
    /// If this function is not called it will default to a [`DefaultCacheValidator`].
    /// The default validator for std builds is the [`TimedCacheValidator`].
    /// The default validator for no_std builds is the [`CountCacheValidator`].
    ///
    /// The default setting is `DefaultCacheValidator::default()`.
    ///
    /// # Examples:
    ///
    /// ```
    /// # const MAGIC_VALUE: u64 = 0x23bd_318f_f3a3_5821;
    /// use memflow::prelude::v1::*;
    /// use memflow::dummy::DummyMemory;
    /// use std::time::Duration;
    /// # use memflow::dummy::DummyOs;
    /// # use memflow::architecture::x86::x64;
    ///
    /// # let phys_mem = DummyMemory::new(size::mb(16));
    /// # let mut os = DummyOs::new(phys_mem);
    /// # let (dtb, virt_base) = os.alloc_dtb(size::mb(8), &[]);
    /// # let phys_mem = os.into_inner();
    /// # let translator = x64::new_translator(dtb);
    /// let mut virt_mem = VirtualDma::new(phys_mem, x64::ARCH, translator);
    ///
    /// let mut cached_mem = CachedView::builder(virt_mem)
    ///     .arch(x64::ARCH)
    ///     .validator(DefaultCacheValidator::new(Duration::from_millis(2000).into()))
    ///     .build()
    ///     .unwrap();
    ///
    /// let addr = virt_base; // some arbitrary address
    ///
    /// cached_mem.write(addr, &MAGIC_VALUE).unwrap();
    ///
    /// let value: u64 = cached_mem.read(addr).unwrap();
    /// assert_eq!(value, MAGIC_VALUE);
    /// ```
    pub fn validator<QN: CacheValidator>(self, validator: QN) -> CachedViewBuilder<T, QN> {
        CachedViewBuilder {
            mem: self.mem,
            validator,
            page_size: self.page_size,
            cache_size: self.cache_size,
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
    /// # const MAGIC_VALUE: u64 = 0x23bd_318f_f3a3_5821;
    /// use memflow::prelude::v1::*;
    /// use memflow::dummy::DummyMemory;
    /// # use memflow::dummy::DummyOs;
    /// # use memflow::architecture::x86::x64;
    ///
    /// # let phys_mem = DummyMemory::new(size::mb(16));
    /// # let mut os = DummyOs::new(phys_mem);
    /// # let (dtb, virt_base) = os.alloc_dtb(size::mb(8), &[]);
    /// # let phys_mem = os.into_inner();
    /// # let translator = x64::new_translator(dtb);
    /// let mut virt_mem = VirtualDma::new(phys_mem, x64::ARCH, translator);
    ///
    /// let mut cached_mem = CachedView::builder(virt_mem)
    ///     .page_size(size::kb(4))
    ///     .build()
    ///     .unwrap();
    ///
    /// let addr = virt_base; // some arbitrary address
    ///
    /// cached_mem.write(addr, &MAGIC_VALUE).unwrap();
    ///
    /// let value: u64 = cached_mem.read(addr).unwrap();
    /// assert_eq!(value, MAGIC_VALUE);
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
    /// # const MAGIC_VALUE: u64 = 0x23bd_318f_f3a3_5821;
    /// use memflow::prelude::v1::*;
    /// use memflow::dummy::DummyMemory;
    /// # use memflow::dummy::DummyOs;
    /// # use memflow::architecture::x86::x64;
    ///
    /// # let phys_mem = DummyMemory::new(size::mb(16));
    /// # let mut os = DummyOs::new(phys_mem);
    /// # let (dtb, virt_base) = os.alloc_dtb(size::mb(8), &[]);
    /// # let phys_mem = os.into_inner();
    /// # let translator = x64::new_translator(dtb);
    /// let mut virt_mem = VirtualDma::new(phys_mem, x64::ARCH, translator);
    ///
    /// let mut cached_mem = CachedView::builder(virt_mem)
    ///     .arch(x64::ARCH)
    ///     .build()
    ///     .unwrap();
    ///
    /// let addr = virt_base; // some arbitrary address
    ///
    /// cached_mem.write(addr, &MAGIC_VALUE).unwrap();
    ///
    /// let value: u64 = cached_mem.read(addr).unwrap();
    /// assert_eq!(value, MAGIC_VALUE);
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
    /// # const MAGIC_VALUE: u64 = 0x23bd_318f_f3a3_5821;
    /// use memflow::prelude::v1::*;
    /// use memflow::dummy::DummyMemory;
    /// # use memflow::dummy::DummyOs;
    /// # use memflow::architecture::x86::x64;
    ///
    /// # let phys_mem = DummyMemory::new(size::mb(16));
    /// # let mut os = DummyOs::new(phys_mem);
    /// # let (dtb, virt_base) = os.alloc_dtb(size::mb(8), &[]);
    /// # let phys_mem = os.into_inner();
    /// # let translator = x64::new_translator(dtb);
    /// let mut virt_mem = VirtualDma::new(phys_mem, x64::ARCH, translator);
    ///
    /// let mut cached_mem = CachedView::builder(virt_mem)
    ///     .arch(x64::ARCH)
    ///     .cache_size(size::mb(2))
    ///     .build()
    ///     .unwrap();
    ///
    /// let addr = virt_base; // some arbitrary address
    ///
    /// cached_mem.write(addr, &MAGIC_VALUE).unwrap();
    ///
    /// let value: u64 = cached_mem.read(addr).unwrap();
    /// assert_eq!(value, MAGIC_VALUE);
    /// ```
    pub fn cache_size(mut self, cache_size: usize) -> Self {
        self.cache_size = cache_size;
        self
    }
}
