use crate::error::Result;

use crate::architecture::Architecture;
use crate::iter::{PageChunks, SplitAtIndex};
use crate::mem::cache::{CacheValidator, TLBCache};
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

        let mut hitc = 0;
        let mut misc = 0;

        let page_size = self.arch.page_size();
        let mut addrs = addrs
            .flat_map(|(addr, buf)| {
                buf.page_chunks_by(addr, page_size, |addr, split, _| {
                    tlb.try_entry(dtb, addr + split.length(), page_size)
                        .is_some()
                        || tlb.try_entry(dtb, addr, page_size).is_some()
                })
            })
            .filter_map(|(addr, buf)| {
                if let Some(entry) = tlb.try_entry(dtb, addr, page_size) {
                    hitc += 1;
                    debug_assert!(buf.length() <= page_size);
                    match entry {
                        Ok(entry) => out.extend(Some((Ok(entry.phys_addr), addr, buf)).into_iter()),
                        Err(error) => out.extend(Some((Err(error), addr, buf)).into_iter()),
                    }
                    None
                } else {
                    misc += core::cmp::max(1, buf.length().as_usize() / page_size.as_usize());
                    Some((addr, buf))
                }
            })
            .peekable();

        if addrs.peek().is_some() {
            self.vat
                .virt_to_phys_iter(phys_mem, dtb, addrs, &mut uncached_out);

            out.extend(uncached_out.into_iter().inspect(|(ret, addr, buf)| {
                if let Ok(paddr) = ret {
                    self.tlb.cache_entry(dtb, *addr, *paddr, page_size);
                } else {
                    self.tlb
                        .cache_invalid_if_uncached(dtb, *addr, buf.length(), page_size);
                }
            }));

            self.hitc += hitc;
            self.misc += misc;
        }
    }
}
