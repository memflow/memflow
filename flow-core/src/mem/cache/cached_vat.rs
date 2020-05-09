use crate::error::Result;

use crate::address::{Address, Page, PhysicalAddress};
use crate::arch::Architecture;
use crate::mem::cache::TLBCache;
use crate::mem::AccessPhysicalMemory;
use crate::vat;
use crate::vat::VirtualAddressTranslator;
use std::ops::Deref;
use std::ops::DerefMut;

#[derive(AccessVirtualMemory)]
pub struct CachedVAT<T: AccessPhysicalMemory + VirtualAddressTranslator, Q: TLBCache> {
    mem: T,
    tlb: Q,
}

impl<T: AccessPhysicalMemory + VirtualAddressTranslator, Q: TLBCache> CachedVAT<T, Q> {
    pub fn with(mem: T, tlb: Q) -> Self {
        Self { mem, tlb }
    }
}

impl<T: AccessPhysicalMemory + VirtualAddressTranslator, Q: TLBCache> VirtualAddressTranslator
    for CachedVAT<T, Q>
{
    fn virt_to_phys(
        &mut self,
        arch: Architecture,
        dtb: Address,
        vaddr: Address,
    ) -> Result<PhysicalAddress> {
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

impl<T: AccessPhysicalMemory + VirtualAddressTranslator, Q: TLBCache> AsMut<T> for CachedVAT<T, Q> {
    fn as_mut(&mut self) -> &mut T {
        &mut self.mem
    }
}

impl<T: AccessPhysicalMemory + VirtualAddressTranslator, Q: TLBCache> Deref for CachedVAT<T, Q> {
    type Target = T;
    fn deref(&self) -> &T {
        &self.mem
    }
}

impl<T: AccessPhysicalMemory + VirtualAddressTranslator, Q: TLBCache> DerefMut for CachedVAT<T, Q> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.mem
    }
}
/*impl<T: AccessPhysicalMemory + VirtualAddressTranslator, Q: TLBCache> core::ops::DerefMut for CachedVAT<T, Q> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.mem
    }
}*/

impl<T: AccessPhysicalMemory + VirtualAddressTranslator, Q: TLBCache> AccessPhysicalMemory
    for CachedVAT<T, Q>
{
    fn phys_read_raw_into(&mut self, addr: PhysicalAddress, out: &mut [u8]) -> Result<()> {
        self.mem.phys_read_raw_into(addr, out)
    }

    fn phys_write_raw(&mut self, addr: PhysicalAddress, data: &[u8]) -> Result<()> {
        self.mem.phys_write_raw(addr, data)
    }
}
