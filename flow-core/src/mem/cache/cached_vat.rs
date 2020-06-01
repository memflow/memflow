use crate::error::Result;

use crate::architecture::Architecture;
use crate::mem::cache::{CacheValidator, TLBCache};
use crate::mem::{PhysicalMemory, PhysicalReadIterator, PhysicalWriteIterator};
use crate::types::{Address, Page, PhysicalAddress};
use crate::vat;
use crate::vat::VirtualAddressTranslatorRaw;
use bumpalo::{collections::Vec as BumpVec, Bump};

#[derive(VirtualMemoryRaw)]
pub struct CachedVAT<T: PhysicalMemory + VirtualAddressTranslatorRaw, Q: CacheValidator> {
    mem: T,
    tlb: TLBCache<Q>,
    arena: Bump,
}

impl<T: PhysicalMemory + VirtualAddressTranslatorRaw, Q: CacheValidator> CachedVAT<T, Q> {
    pub fn with(mem: T, tlb: TLBCache<Q>) -> Self {
        Self {
            mem,
            tlb,
            arena: Bump::new(),
        }
    }
}

impl<T: PhysicalMemory + VirtualAddressTranslatorRaw, Q: CacheValidator> VirtualAddressTranslatorRaw
    for CachedVAT<T, Q>
{
    fn virt_to_phys_iter<
        B,
        VI: Iterator<Item = (Address, B)>,
        OV: Extend<(Result<PhysicalAddress>, Address, B)>,
    >(
        &mut self,
        arch: Architecture,
        dtb: Address,
        addrs: VI,
        out: &mut OV,
    ) {
        self.tlb.validator.update_validity();
        self.arena.reset();

        let tlb = &mut self.tlb;
        let mut uncached_out = BumpVec::new_in(&self.arena);

        let mut addrs = addrs
            .filter_map(|(addr, buf)| {
                if let Some(entry) = tlb.try_entry(dtb, addr, arch.page_size()) {
                    out.extend(Some((Ok(entry.phys_addr), addr, buf)).into_iter());
                    None
                } else {
                    Some((addr, buf))
                }
            })
            .peekable();

        if addrs.peek().is_some() {
            arch.virt_to_phys_iter(&mut self.mem, dtb, addrs, &mut uncached_out);
            out.extend(uncached_out.into_iter().inspect(|(ret, addr, _)| {
                if let Ok(paddr) = ret {
                    self.tlb.cache_entry(dtb, *addr, *paddr, arch.page_size());
                }
            }));
        }
    }
}

impl<T: PhysicalMemory + VirtualAddressTranslatorRaw, Q: CacheValidator> PhysicalMemory
    for CachedVAT<T, Q>
{
    fn phys_read_iter<'b, PI: PhysicalReadIterator<'b>>(&'b mut self, iter: PI) -> Result<()> {
        self.mem.phys_read_iter(iter)
    }

    fn phys_write_iter<'b, PI: PhysicalWriteIterator<'b>>(&'b mut self, iter: PI) -> Result<()> {
        self.mem.phys_write_iter(iter)
    }
}
