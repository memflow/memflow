use crate::error::Result;

use crate::architecture::Architecture;
use crate::iter::SplitAtIndex;
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
}

impl<V: VirtualTranslate, Q: CacheValidator> CachedVirtualTranslate<V, Q> {
    pub fn with(vat: V, tlb: TLBCache<Q>, arch: Architecture) -> Self {
        Self {
            vat,
            tlb,
            arch,
            arena: Bump::new(),
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
        //self.tlb.validator.update_validity();
        self.arena.reset();

        let tlb = &mut self.tlb;
        let mut uncached_out = BumpVec::new_in(&self.arena);

        let page_size = self.arch.page_size();
        let mut addrs = addrs
            .filter_map(|(addr, buf)| {
                if let Some(entry) = tlb.try_entry(dtb, addr, page_size) {
                    out.extend(Some((Ok(entry.phys_addr), addr, buf)).into_iter());
                    None
                } else {
                    Some((addr, buf))
                }
            })
            .peekable();

        if addrs.peek().is_some() {
            self.vat
                .virt_to_phys_iter(phys_mem, dtb, addrs, &mut uncached_out);
            //Do not actually cache anything, because TLB does not support recent VTOP optimization
            out.extend(uncached_out.into_iter());
            /*out.extend(uncached_out.into_iter().inspect(|(ret, addr, _)| {
                if let Ok(paddr) = ret {
                    self.tlb.cache_entry(dtb, *addr, *paddr, page_size);
                }
            }));*/
        }
    }
}
