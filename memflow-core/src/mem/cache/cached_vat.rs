use crate::error::{Error, Result};

use super::tlb_cache::TLBCache;
use crate::architecture::{Architecture, ScopedVirtualTranslate};
use crate::iter::{PageChunks, SplitAtIndex};
use crate::mem::cache::{CacheValidator, TimedCacheValidator};
use crate::mem::virt_translate::VirtualTranslate;
use crate::mem::PhysicalMemory;
use crate::types::{Address, PhysicalAddress};

use bumpalo::{collections::Vec as BumpVec, Bump};

pub struct CachedVirtualTranslate<V, Q> {
    vat: V,
    tlb: TLBCache<Q>,
    arch: &'static dyn Architecture,
    arena: Bump,
    pub hitc: usize,
    pub misc: usize,
}

impl<V: VirtualTranslate, Q: CacheValidator> CachedVirtualTranslate<V, Q> {
    pub fn new(vat: V, tlb: TLBCache<Q>, arch: &'static dyn Architecture) -> Self {
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

impl<V: VirtualTranslate> CachedVirtualTranslate<V, TimedCacheValidator> {
    pub fn builder(vat: V) -> CachedVirtualTranslateBuilder<V, TimedCacheValidator> {
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
    arch: Option<&'static dyn Architecture>,
}

impl<V: VirtualTranslate> CachedVirtualTranslateBuilder<V, TimedCacheValidator> {
    fn new(vat: V) -> Self {
        Self {
            vat,
            validator: TimedCacheValidator::default(),
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

    pub fn arch(mut self, arch: &'static dyn Architecture) -> Self {
        self.arch = Some(arch);
        self
    }
}
