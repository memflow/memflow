use crate::error::Result;

use super::tlb_cache::TLBCache;
use crate::architecture::Architecture;
use crate::iter::{PageChunks, SplitAtIndex};
use crate::mem::cache::CacheValidator;
use crate::mem::virt_translate::VirtualTranslate;
use crate::mem::PhysicalMemory;
use crate::types::{Address, PhysicalAddress};

use bumpalo::{collections::Vec as BumpVec, Bump};

pub struct CachedVirtualTranslate<V: VirtualTranslate, Q: CacheValidator> {
    vat: V,
    tlb: TLBCache<Q>,
    arch: Architecture,
    arena: Bump,
    pub hitc: usize,
    pub misc: usize,
}

impl<V: VirtualTranslate, Q: CacheValidator> CachedVirtualTranslate<V, Q> {
    pub fn builder() -> CachedVirtualTranslateBuilder<V, Q> {
        CachedVirtualTranslateBuilder::default()
    }

    pub fn with(vat: V, tlb: TLBCache<Q>, arch: Architecture) -> Self {
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

impl<V: VirtualTranslate, Q: CacheValidator> VirtualTranslate for CachedVirtualTranslate<V, Q> {
    fn virt_to_phys_iter<T, B, VI, OV>(
        &mut self,
        phys_mem: &mut T,
        dtb: Address,
        addrs: VI,
        out: &mut OV,
    ) where
        T: PhysicalMemory + ?Sized,
        B: SplitAtIndex,
        VI: Iterator<Item = (Address, B)>,
        OV: Extend<(Result<PhysicalAddress>, Address, B)>,
    {
        self.tlb.validator.update_validity();
        self.arena.reset();

        let tlb = &self.tlb;
        let mut uncached_out = BumpVec::new_in(&self.arena);
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
                    tlb.try_entry(dtb, addr + split.length(), arch).is_some()
                        || tlb.try_entry(dtb, addr, arch).is_some()
                })
            })
            .filter_map(|(addr, buf)| {
                if let Some(entry) = tlb.try_entry(dtb, addr, arch) {
                    hitc += 1;
                    debug_assert!(buf.length() <= arch.page_size());
                    match entry {
                        Ok(entry) => out.extend(Some((Ok(entry.phys_addr), addr, buf)).into_iter()),
                        Err(error) => out.extend(Some((Err(error), addr, buf)).into_iter()),
                    }
                    None
                } else {
                    misc += core::cmp::max(1, buf.length() / arch.page_size());
                    Some((addr, buf))
                }
            })
            .peekable();

        if addrs.peek().is_some() {
            self.vat
                .virt_to_phys_iter(phys_mem, dtb, addrs, &mut uncached_out);
        }

        let uncached_iter = uncached_in.into_iter();

        self.vat
            .virt_to_phys_iter(phys_mem, dtb, uncached_iter, out);

        out.extend(uncached_out.into_iter().inspect(|(ret, addr, buf)| {
            if let Ok(paddr) = ret {
                self.tlb.cache_entry(dtb, *addr, *paddr, arch);
            } else {
                self.tlb
                    .cache_invalid_if_uncached(dtb, *addr, buf.length(), arch);
            }
        }));

        self.hitc += hitc;
        self.misc += misc;
    }
}

pub struct CachedVirtualTranslateBuilder<V, Q> {
    vat: Option<V>,
    validator: Option<Q>,
    entries: Option<usize>,
    arch: Option<Architecture>,
}

impl<V: VirtualTranslate, Q: CacheValidator> Default for CachedVirtualTranslateBuilder<V, Q> {
    fn default() -> Self {
        Self {
            vat: None,
            validator: None,
            entries: Some(2048),
            arch: None,
        }
    }
}

impl<V: VirtualTranslate, Q: CacheValidator> CachedVirtualTranslateBuilder<V, Q> {
    pub fn build(self) -> Result<CachedVirtualTranslate<V, Q>> {
        Ok(CachedVirtualTranslate::with(
            self.vat.ok_or("vat must be initialized")?,
            TLBCache::new(
                self.entries.ok_or("entries must be initialized")?,
                self.validator.ok_or("validator must be initialized")?,
            ),
            self.arch.ok_or("arch must be initialized")?,
        ))
    }

    pub fn vat(mut self, vat: V) -> Self {
        self.vat = Some(vat);
        self
    }

    pub fn validator(mut self, validator: Q) -> Self {
        self.validator = Some(validator);
        self
    }

    pub fn entries(mut self, entries: usize) -> Self {
        self.entries = Some(entries);
        self
    }

    pub fn arch(mut self, arch: Architecture) -> Self {
        self.arch = Some(arch);
        self
    }
}
