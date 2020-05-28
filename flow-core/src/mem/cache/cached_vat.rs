use crate::error::Result;

use crate::architecture::Architecture;
use crate::mem::cache::{CacheValidator, TLBCache};
use crate::mem::{AccessPhysicalMemory, PhysicalReadIterator, PhysicalWriteIterator};
use crate::types::{Address, Page, PhysicalAddress};
use crate::vat;
use crate::vat::VirtualAddressTranslator;
use bumpalo::{collections::Vec as BumpVec, Bump};

#[derive(AccessVirtualMemory)]
pub struct CachedVAT<T: AccessPhysicalMemory + VirtualAddressTranslator, Q: CacheValidator> {
    mem: T,
    tlb: TLBCache<Q>,
    arena: Bump,
}

impl<T: AccessPhysicalMemory + VirtualAddressTranslator, Q: CacheValidator> CachedVAT<T, Q> {
    pub fn with(mem: T, tlb: TLBCache<Q>) -> Self {
        Self {
            mem,
            tlb,
            arena: Bump::new(),
        }
    }
}

impl<T: AccessPhysicalMemory + VirtualAddressTranslator, Q: CacheValidator> VirtualAddressTranslator
    for CachedVAT<T, Q>
{
    fn virt_to_phys_iter<B, VI: Iterator<Item = (Address, B)>>(
        &mut self,
        arch: Architecture,
        dtb: Address,
        addrs: VI,
        out: &mut Vec<(Result<PhysicalAddress>, Address, B)>,
    ) {
        self.tlb.validator.update_validity();
        self.arena.reset();

        let tlb = &mut self.tlb;
        let mut cached_out = BumpVec::new_in(&self.arena);

        let mut addrs = addrs
            .filter_map(|(addr, buf)| {
                if let Some(entry) = tlb.try_entry(dtb, addr, arch.page_size()) {
                    cached_out.push((Ok(entry.phys_addr), addr, buf));
                    None
                } else {
                    Some((addr, buf))
                }
            })
            .peekable();

        if addrs.peek().is_some() {
            let last_idx = out.len();
            arch.virt_to_phys_iter(&mut self.mem, dtb, addrs, out);
            for (ret, addr, _) in out.iter_mut().skip(last_idx) {
                if let Ok(ret) = ret {
                    self.tlb
                        .cache_entry(dtb, *addr, ret.page.unwrap(), arch.page_size());
                }
            }
        }

        for x in cached_out.into_iter() {
            out.push(x);
        }
    }
}

impl<T: AccessPhysicalMemory + VirtualAddressTranslator, Q: CacheValidator> AccessPhysicalMemory
    for CachedVAT<T, Q>
{
    fn phys_read_raw_iter<'b, PI: PhysicalReadIterator<'b>>(&'b mut self, iter: PI) -> Result<()> {
        self.mem.phys_read_raw_iter(iter)
    }

    fn phys_write_raw_iter<'b, PI: PhysicalWriteIterator<'b>>(
        &'b mut self,
        iter: PI,
    ) -> Box<dyn PhysicalWriteIterator<'b>> {
        self.mem.phys_write_raw_iter(iter)
    }
}
