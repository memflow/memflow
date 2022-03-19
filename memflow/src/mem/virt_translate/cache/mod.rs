use crate::error::{Error, ErrorKind, ErrorOrigin, Result};

mod tlb_cache;

use crate::architecture::ArchitectureObj;
use crate::iter::{PageChunks, SplitAtIndex};
use crate::mem::virt_translate::VirtualTranslate2;
use crate::mem::PhysicalMemory;
use crate::types::cache::{CacheValidator, DefaultCacheValidator};
use crate::types::{umem, Address};
use cglue::tuple::*;
use tlb_cache::TlbCache;

use super::{VirtualTranslate3, VtopFailureCallback, VtopOutputCallback};

use cglue::callback::FromExtend;

use bumpalo::{collections::Vec as BumpVec, Bump};

/// CachedVirtualTranslate trasnaparently caches virtual addresss translations.
///
/// Using a VAT cache can provide significant speedups, since page table walks perform a number
/// of memory reads, which induces noticeable latency, especially on slow memory backends.
///
/// Using the `builder` function is the recommended way to create such a cache.
///
/// # Examples
///
///
/// ```
/// use memflow::mem::CachedVirtualTranslate;
/// # use memflow::architecture::x86::x64;
/// # use memflow::dummy::{DummyMemory, DummyOs};
/// # use memflow::mem::{DirectTranslate, VirtualDma, MemoryView, VirtualTranslate2};
/// # use memflow::types::size;
/// # let mem = DummyMemory::new(size::mb(32));
/// # let mut os = DummyOs::new(mem);
/// # let virt_size = size::mb(8);
/// # let (dtb, virt_base) = os.alloc_dtb(virt_size, &[]);
/// # let mut mem = os.into_inner();
/// # let translator = x64::new_translator(dtb);
/// # let mut vat = DirectTranslate::new();
/// let mut cached_vat = CachedVirtualTranslate::builder(&mut vat)
///     .arch(x64::ARCH)
///     .build()
///     .unwrap();
/// ```
///
/// Testing that cached translation is at least 2x faster than uncached translation when having a cache hit:
///
/// ```
/// use std::time::{Duration, Instant};
/// # use memflow::mem::CachedVirtualTranslate;
/// # use memflow::architecture::x86::x64;
/// # use memflow::dummy::{DummyMemory, DummyOs};
/// # use memflow::mem::{DirectTranslate, VirtualDma, MemoryView, VirtualTranslate2};
/// # use memflow::types::size;
/// # let mem = DummyMemory::new(size::mb(32));
/// # let mut os = DummyOs::new(mem);
/// # let virt_size = size::mb(8);
/// # let (dtb, virt_base) = os.alloc_dtb(virt_size, &[]);
/// # let mut mem = os.into_inner();
/// # let translator = x64::new_translator(dtb);
/// # let mut vat = DirectTranslate::new();
/// # let mut cached_vat = CachedVirtualTranslate::builder(&mut vat)
/// #     .arch(x64::ARCH)
/// #     .build()
/// #     .unwrap();
///
/// let translation_address = virt_base;
///
/// let iter_count = 1024;
///
/// let avg_cached = (0..iter_count).map(|_| {
///         let timer = Instant::now();
///         cached_vat
///             .virt_to_phys(&mut mem, &translator, translation_address)
///             .unwrap();
///         timer.elapsed()
///     })
///     .sum::<Duration>() / iter_count;
///
/// println!("{:?}", avg_cached);
///
/// std::mem::drop(cached_vat);
///
/// let avg_uncached = (0..iter_count).map(|_| {
///         let timer = Instant::now();
///         vat
///             .virt_to_phys(&mut mem, &translator, translation_address)
///             .unwrap();
///         timer.elapsed()
///     })
///     .sum::<Duration>() / iter_count;
///
/// println!("{:?}", avg_uncached);
///
/// assert!(avg_cached * 9 <= avg_uncached * 7);
/// ```
pub struct CachedVirtualTranslate<V, Q> {
    vat: V,
    tlb: TlbCache<Q>,
    arch: ArchitectureObj,
    arena: Bump,
    pub hitc: umem,
    pub misc: umem,
}

impl<V: VirtualTranslate2, Q: CacheValidator> CachedVirtualTranslate<V, Q> {
    pub fn new(vat: V, tlb: TlbCache<Q>, arch: ArchitectureObj) -> Self {
        Self {
            vat,
            tlb,
            arch,
            arena: Bump::new(),
            hitc: 0,
            misc: 0,
        }
    }
}

impl<V: VirtualTranslate2> CachedVirtualTranslate<V, DefaultCacheValidator> {
    pub fn builder(vat: V) -> CachedVirtualTranslateBuilder<V, DefaultCacheValidator> {
        CachedVirtualTranslateBuilder::new(vat)
    }
}

impl<V: VirtualTranslate2 + Clone, Q: CacheValidator + Clone> Clone
    for CachedVirtualTranslate<V, Q>
{
    fn clone(&self) -> Self {
        Self {
            vat: self.vat.clone(),
            tlb: self.tlb.clone(),
            arch: self.arch,
            arena: Bump::new(),
            hitc: self.hitc,
            misc: self.misc,
        }
    }
}

impl<V: VirtualTranslate2, Q: CacheValidator> VirtualTranslate2 for CachedVirtualTranslate<V, Q> {
    fn virt_to_phys_iter<T, B, D, VI>(
        &mut self,
        phys_mem: &mut T,
        translator: &D,
        addrs: VI,
        out: &mut VtopOutputCallback<B>,
        out_fail: &mut VtopFailureCallback<B>,
    ) where
        T: PhysicalMemory + ?Sized,
        B: SplitAtIndex,
        D: VirtualTranslate3,
        VI: Iterator<Item = CTup3<Address, Address, B>>,
    {
        self.tlb.validator.update_validity();
        self.arena.reset();

        let tlb = &mut self.tlb;
        let vat = &mut self.vat;
        let mut uncached_out = BumpVec::new_in(&self.arena);
        let mut uncached_out_fail = BumpVec::new_in(&self.arena);
        let mut uncached_in = BumpVec::new_in(&self.arena);

        let mut hitc = 0;
        let mut misc = 0;

        let arch = self.arch;
        let mut addrs = addrs
            .filter_map(|CTup3(addr, meta_addr, buf)| {
                if tlb.is_read_too_long(arch, buf.length() as umem) {
                    uncached_in.push(CTup3(addr, meta_addr, buf));
                    None
                } else {
                    Some((addr, meta_addr, buf))
                }
            })
            .flat_map(|(addr, meta_addr, buf)| {
                (meta_addr, buf).page_chunks_by(addr, arch.page_size(), |addr, (_, split), _| {
                    tlb.try_entry(translator, addr + split.length(), arch)
                        .is_some()
                        || tlb.try_entry(translator, addr, arch).is_some()
                })
            })
            .filter_map(|(addr, (meta_addr, buf))| {
                if let Some(entry) = tlb.try_entry(translator, addr, arch) {
                    hitc += 1;
                    debug_assert!(buf.length() <= arch.page_size() as umem);
                    // TODO: handle case
                    let _ = match entry {
                        Ok(entry) => out.call(CTup3(entry.phys_addr, meta_addr, buf)),
                        Err(error) => out_fail.call((error, CTup3(addr, meta_addr, buf))),
                    };
                    None
                } else {
                    misc += core::cmp::max(1, buf.length() / arch.page_size() as umem);
                    Some(CTup3(addr, meta_addr, (addr, buf)))
                }
            })
            .peekable();

        if addrs.peek().is_some() {
            vat.virt_to_phys_iter(
                phys_mem,
                translator,
                addrs,
                &mut uncached_out.from_extend(),
                &mut uncached_out_fail.from_extend(),
            );
        }

        let mut uncached_iter = uncached_in.into_iter().peekable();

        if uncached_iter.peek().is_some() {
            vat.virt_to_phys_iter(phys_mem, translator, uncached_iter, out, out_fail);
        }

        out.extend(
            uncached_out
                .into_iter()
                .map(|CTup3(paddr, meta_addr, (addr, buf))| {
                    tlb.cache_entry(translator, addr, paddr, arch);
                    CTup3(paddr, meta_addr, buf)
                }),
        );

        out_fail.extend(uncached_out_fail.into_iter().map(
            |(err, CTup3(vaddr, meta_addr, (_, buf)))| {
                tlb.cache_invalid_if_uncached(translator, vaddr, buf.length() as umem, arch);
                (err, CTup3(vaddr, meta_addr, buf))
            },
        ));

        self.hitc += hitc;
        self.misc += misc;
    }
}

pub struct CachedVirtualTranslateBuilder<V, Q> {
    vat: V,
    validator: Q,
    entries: Option<usize>,
    arch: Option<ArchitectureObj>,
}

impl<V: VirtualTranslate2> CachedVirtualTranslateBuilder<V, DefaultCacheValidator> {
    fn new(vat: V) -> Self {
        Self {
            vat,
            validator: DefaultCacheValidator::default(),
            entries: Some(2048),
            arch: None,
        }
    }
}

impl<V: VirtualTranslate2, Q: CacheValidator> CachedVirtualTranslateBuilder<V, Q> {
    pub fn build(self) -> Result<CachedVirtualTranslate<V, Q>> {
        Ok(CachedVirtualTranslate::new(
            self.vat,
            TlbCache::new(
                self.entries.ok_or_else(|| {
                    Error(ErrorOrigin::Cache, ErrorKind::Uninitialized)
                        .log_error("entries must be initialized")
                })?,
                self.validator,
            ),
            self.arch.ok_or_else(|| {
                Error(ErrorOrigin::Cache, ErrorKind::Uninitialized)
                    .log_error("arch must be initialized")
            })?,
        ))
    }

    pub fn validator<QN: CacheValidator>(
        self,
        validator: QN,
    ) -> CachedVirtualTranslateBuilder<V, QN> {
        CachedVirtualTranslateBuilder {
            vat: self.vat,
            validator,
            entries: self.entries,
            arch: self.arch,
        }
    }

    pub fn entries(mut self, entries: usize) -> Self {
        self.entries = Some(entries);
        self
    }

    pub fn arch(mut self, arch: impl Into<ArchitectureObj>) -> Self {
        self.arch = Some(arch.into());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::architecture::x86;
    use crate::dummy::{DummyMemory, DummyOs};
    use crate::error::PartialResultExt;
    use crate::mem::{DirectTranslate, PhysicalMemory};
    use crate::mem::{MemoryView, VirtualDma};
    use crate::types::cache::timed_validator::TimedCacheValidator;
    use crate::types::{size, Address};

    use coarsetime::Duration;

    fn build_mem(
        buf: &[u8],
    ) -> (
        impl PhysicalMemory,
        impl MemoryView + Clone,
        Address,
        Address,
    ) {
        let mem = DummyMemory::new(buf.len() + size::mb(2));
        let (os, dtb, virt_base) = DummyOs::new_and_dtb(mem, buf.len(), buf);
        let translator = x86::x64::new_translator(dtb);

        let vat = CachedVirtualTranslate::builder(DirectTranslate::new())
            .arch(x86::x64::ARCH)
            .validator(TimedCacheValidator::new(Duration::from_secs(100)))
            .entries(2048)
            .build()
            .unwrap();

        let mem = os.into_inner();

        let vmem = VirtualDma::with_vat(mem.clone(), x86::x64::ARCH, translator, vat);

        (mem, vmem, virt_base, dtb)
    }

    fn standard_buffer(size: usize) -> Vec<u8> {
        (0..size)
            .step_by(std::mem::size_of_val(&size))
            .flat_map(|v| v.to_le_bytes().to_vec())
            .collect()
    }

    #[test]
    fn valid_after_pt_destruction() {
        // The following test is against volatility of the page tables
        // Given that the cache is valid for 100 seconds, this test should
        // pass without a single entry becoming invalid.
        let buffer = standard_buffer(size::mb(2));
        let (mut mem, mut vmem, virt_base, dtb) = build_mem(&buffer);

        let mut read_into = vec![0u8; size::mb(2)];

        vmem.read_raw_into(virt_base, &mut read_into)
            .data()
            .unwrap();

        assert!(read_into == buffer);

        // Destroy the page tables
        mem.phys_write(dtb.into(), vec![0u8; size::kb(4)].as_slice())
            .unwrap();

        vmem.read_raw_into(virt_base, &mut read_into)
            .data()
            .unwrap();
        assert!(read_into == buffer);

        // Also test that cloning of the entries works as it is supposed to
        let mut vmem_cloned = vmem.clone();

        vmem_cloned
            .read_raw_into(virt_base, &mut read_into)
            .data()
            .unwrap();
        assert!(read_into == buffer);

        vmem.read_raw_into(virt_base, &mut read_into)
            .data()
            .unwrap();
        assert!(read_into == buffer);
    }
}
