use crate::error::{Error, Result};

use super::tlb_cache::TLBCache;
use crate::architecture::{ArchitectureObj, ScopedVirtualTranslate};
use crate::iter::{PageChunks, SplitAtIndex};
use crate::mem::cache::{CacheValidator, DefaultCacheValidator};
use crate::mem::virt_translate::VirtualTranslate;
use crate::mem::PhysicalMemory;
use crate::types::{Address, PhysicalAddress};

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
/// use memflow::mem::cache::CachedVirtualTranslate;
/// # use memflow::architecture::x86::x64;
/// # use memflow::mem::dummy::DummyMemory;
/// # use memflow::mem::{DirectTranslate, VirtualDMA, VirtualMemory, VirtualTranslate};
/// # use memflow::types::size;
/// # let mut mem = DummyMemory::new(size::mb(32));
/// # let virt_size = size::mb(8);
/// # let (dtb, virt_base) = mem.alloc_dtb(virt_size, &[]);
/// # let translator = x64::new_translator(dtb);
/// # let mut vat = DirectTranslate::new();
/// let mut cached_vat = CachedVirtualTranslate::builder(&mut vat)
///     .arch(x64::ARCH)
///     .build()
///     .unwrap();
/// ```
///
/// Testing that cached translation is 4x faster than uncached translation when having a cache hit:
///
/// ```
/// use std::time::{Duration, Instant};
/// # use memflow::mem::cache::CachedVirtualTranslate;
/// # use memflow::architecture::x86::x64;
/// # use memflow::mem::dummy::DummyMemory;
/// # use memflow::mem::{DirectTranslate, VirtualDMA, VirtualMemory, VirtualTranslate};
/// # use memflow::types::size;
/// # let mut mem = DummyMemory::new(size::mb(32));
/// # let virt_size = size::mb(8);
/// # let (dtb, virt_base) = mem.alloc_dtb(virt_size, &[]);
/// # let translator = x64::new_translator(dtb);
/// # let mut vat = DirectTranslate::new();
/// # let mut cached_vat = CachedVirtualTranslate::builder(&mut vat)
/// #     .arch(x64::ARCH)
/// #     .build()
/// #     .unwrap();
///
/// let translation_address = virt_base;
///
/// let iter_count = 512;
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
/// assert!(avg_cached * 4 <= avg_uncached);
/// ```
pub struct CachedVirtualTranslate<V, Q> {
    vat: V,
    tlb: TLBCache<Q>,
    arch: ArchitectureObj,
    arena: Bump,
    pub hitc: usize,
    pub misc: usize,
}

impl<V: VirtualTranslate, Q: CacheValidator> CachedVirtualTranslate<V, Q> {
    pub fn new(vat: V, tlb: TLBCache<Q>, arch: ArchitectureObj) -> Self {
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

impl<V: VirtualTranslate> CachedVirtualTranslate<V, DefaultCacheValidator> {
    pub fn builder(vat: V) -> CachedVirtualTranslateBuilder<V, DefaultCacheValidator> {
        CachedVirtualTranslateBuilder::new(vat)
    }
}

impl<V: VirtualTranslate + Clone, Q: CacheValidator + Clone> Clone
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

impl<V: VirtualTranslate, Q: CacheValidator> VirtualTranslate for CachedVirtualTranslate<V, Q> {
    fn virt_to_phys_iter<T, B, D, VI, VO, FO>(
        &mut self,
        phys_mem: &mut T,
        translator: &D,
        addrs: VI,
        out: &mut VO,
        out_fail: &mut FO,
    ) where
        T: PhysicalMemory + ?Sized,
        B: SplitAtIndex,
        D: ScopedVirtualTranslate,
        VI: Iterator<Item = (Address, B)>,
        VO: Extend<(PhysicalAddress, B)>,
        FO: Extend<(Error, Address, B)>,
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
            .filter_map(|(addr, buf)| {
                if tlb.is_read_too_long(arch, buf.length()) {
                    uncached_in.push((addr, buf));
                    None
                } else {
                    Some((addr, buf))
                }
            })
            .flat_map(|(addr, buf)| {
                buf.page_chunks_by(addr, arch.page_size(), |addr, split, _| {
                    tlb.try_entry(translator, addr + split.length(), arch)
                        .is_some()
                        || tlb.try_entry(translator, addr, arch).is_some()
                })
            })
            .filter_map(|(addr, buf)| {
                if let Some(entry) = tlb.try_entry(translator, addr, arch) {
                    hitc += 1;
                    debug_assert!(buf.length() <= arch.page_size());
                    match entry {
                        Ok(entry) => out.extend(Some((entry.phys_addr, buf))),
                        Err(error) => out_fail.extend(Some((error, addr, buf))),
                    }
                    None
                } else {
                    misc += core::cmp::max(1, buf.length() / arch.page_size());
                    Some((addr, (addr, buf)))
                }
            })
            .peekable();

        if addrs.peek().is_some() {
            vat.virt_to_phys_iter(
                phys_mem,
                translator,
                addrs,
                &mut uncached_out,
                &mut uncached_out_fail,
            );
        }

        let mut uncached_iter = uncached_in.into_iter().peekable();

        if uncached_iter.peek().is_some() {
            vat.virt_to_phys_iter(phys_mem, translator, uncached_iter, out, out_fail);
        }

        out.extend(uncached_out.into_iter().map(|(paddr, (addr, buf))| {
            tlb.cache_entry(translator, addr, paddr, arch);
            (paddr, buf)
        }));

        out_fail.extend(uncached_out_fail.into_iter().map(|(err, vaddr, (_, buf))| {
            tlb.cache_invalid_if_uncached(translator, vaddr, buf.length(), arch);
            (err, vaddr, buf)
        }));

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

impl<V: VirtualTranslate> CachedVirtualTranslateBuilder<V, DefaultCacheValidator> {
    fn new(vat: V) -> Self {
        Self {
            vat,
            validator: DefaultCacheValidator::default(),
            entries: Some(2048),
            arch: None,
        }
    }
}

impl<V: VirtualTranslate, Q: CacheValidator> CachedVirtualTranslateBuilder<V, Q> {
    pub fn build(self) -> Result<CachedVirtualTranslate<V, Q>> {
        Ok(CachedVirtualTranslate::new(
            self.vat,
            TLBCache::new(
                self.entries.ok_or("entries must be initialized")?,
                self.validator,
            ),
            self.arch.ok_or("arch must be initialized")?,
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

    pub fn arch(mut self, arch: ArchitectureObj) -> Self {
        self.arch = Some(arch);
        self
    }
}

#[cfg(test)]
mod tests {
    use crate::architecture::x86;

    use crate::error::PartialResultExt;
    use crate::mem::cache::cached_vat::CachedVirtualTranslate;
    use crate::mem::cache::timed_validator::TimedCacheValidator;
    use crate::mem::{dummy::DummyMemory, DirectTranslate, PhysicalMemory};
    use crate::mem::{VirtualDMA, VirtualMemory};
    use crate::types::{size, Address};
    use coarsetime::Duration;

    fn build_mem(
        buf: &[u8],
    ) -> (
        impl PhysicalMemory,
        impl VirtualMemory + Clone,
        Address,
        Address,
    ) {
        let (mem, dtb, virt_base) =
            DummyMemory::new_and_dtb(buf.len() + size::mb(2), buf.len(), buf);
        let translator = x86::x64::new_translator(dtb);

        let vat = CachedVirtualTranslate::builder(DirectTranslate::new())
            .arch(x86::x64::ARCH)
            .validator(TimedCacheValidator::new(Duration::from_secs(100)))
            .entries(2048)
            .build()
            .unwrap();
        let vmem = VirtualDMA::with_vat(mem.clone(), x86::x64::ARCH, translator, vat);

        (mem, vmem, virt_base, dtb)
    }

    fn standard_buffer(size: usize) -> Vec<u8> {
        (0..size)
            .step_by(std::mem::size_of_val(&size))
            .flat_map(|v| v.to_le_bytes().iter().copied().collect::<Vec<u8>>())
            .collect()
    }

    #[test]
    fn valid_after_pt_destruction() {
        // The following test is against volatility of the page tables
        // Given that the cache is valid for 100 seconds, this test should
        // pass without a single entry becoming invalid.
        let buffer = standard_buffer(size::mb(2));
        let (mut mem, mut vmem, virt_base, dtb) = build_mem(&buffer);

        let mut read_into = vec![0; size::mb(2)];
        vmem.virt_read_raw_into(virt_base, &mut read_into)
            .data()
            .unwrap();
        assert!(read_into == buffer);

        // Destroy the page tables
        mem.phys_write_raw(dtb.into(), &vec![0; size::kb(4)])
            .unwrap();

        vmem.virt_read_raw_into(virt_base, &mut read_into)
            .data()
            .unwrap();
        assert!(read_into == buffer);

        // Also test that cloning of the entries works as it is supposed to
        let mut vmem_cloned = vmem.clone();

        vmem_cloned
            .virt_read_raw_into(virt_base, &mut read_into)
            .data()
            .unwrap();
        assert!(read_into == buffer);

        vmem.virt_read_raw_into(virt_base, &mut read_into)
            .data()
            .unwrap();
        assert!(read_into == buffer);
    }
}
