use crate::error::Result;

use crate::architecture::Architecture;
use crate::mem::cache::{CacheValidator, TLBCache};
use crate::mem::{AccessPhysicalMemory, PhysicalReadIterator, PhysicalWriteIterator};
use crate::types::{Address, Page, PhysicalAddress};
use crate::vat;
use crate::vat::VirtualAddressTranslator;

#[derive(AccessVirtualMemory)]
pub struct CachedVAT<T: AccessPhysicalMemory + VirtualAddressTranslator, Q: CacheValidator> {
    mem: T,
    tlb: TLBCache<Q>,
}

impl<T: AccessPhysicalMemory + VirtualAddressTranslator, Q: CacheValidator> CachedVAT<T, Q> {
    pub fn with(mem: T, tlb: TLBCache<Q>) -> Self {
        Self { mem, tlb }
    }
}

impl<T: AccessPhysicalMemory + VirtualAddressTranslator, Q: CacheValidator> VirtualAddressTranslator
    for CachedVAT<T, Q>
{
    fn virt_to_phys(
        &mut self,
        arch: Architecture,
        dtb: Address,
        vaddr: Address,
    ) -> Result<PhysicalAddress> {
        self.tlb.validator.update_validity();
        if let Some(entry) = self.tlb.try_entry(dtb, vaddr, arch.page_size()) {
            Ok(entry.phys_addr)
        } else {
            let ret = arch.virt_to_phys(&mut self.mem, dtb, vaddr)?;
            self.tlb
                .cache_entry(dtb, vaddr, ret.page.unwrap(), arch.page_size());
            Ok(ret)
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
