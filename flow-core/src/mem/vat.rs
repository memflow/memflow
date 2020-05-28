#[cfg(test)]
mod tests;

use crate::error::{Error, Result};

use crate::architecture::Architecture;
use crate::iter::page_chunks::{PageChunks, PageChunksMut};
use crate::mem::{
    virt::{VirtualReadIterator, VirtualWriteIterator},
    AccessPhysicalMemory,
};
use crate::types::{Address, Page, PhysicalAddress};

pub trait VirtualAddressTranslator {
    fn virt_to_phys_iter<B, VI: Iterator<Item = (Address, B)>>(
        &mut self,
        arch: Architecture,
        dtb: Address,
        addrs: VI,
        out: &mut Vec<(Result<PhysicalAddress>, Address, B)>,
    );

    fn virt_to_phys(
        &mut self,
        arch: Architecture,
        dtb: Address,
        vaddr: Address,
    ) -> Result<PhysicalAddress> {
        let mut out = Vec::with_capacity(1);
        self.virt_to_phys_iter(arch, dtb, Some((vaddr, false)).into_iter(), &mut out);
        out.pop().unwrap().0
    }
}

pub fn virt_read_raw_iter<
    'a,
    T: AccessPhysicalMemory + VirtualAddressTranslator,
    VI: VirtualReadIterator<'a>,
>(
    mem: &mut T,
    arch: Architecture,
    dtb: Address,
    iter: VI,
) -> Result<()> {
    //30% perf hit on dummy!!! FIXME!!!
    let mut translation = Vec::with_capacity(iter.size_hint().0);
    mem.virt_to_phys_iter(
        arch,
        dtb,
        iter.flat_map(|(addr, out)| PageChunksMut::create_from(out, addr, arch.page_size())),
        &mut translation,
    );

    let iter = translation.into_iter().filter_map(|(paddr, _, out)| {
        if let Ok(paddr) = paddr {
            Some((paddr, out))
        } else {
            for v in out.iter_mut() {
                *v = 0
            }
            None
        }
    });

    mem.phys_read_raw_iter(iter)?;

    Ok(())
}

#[allow(unused)]
pub fn virt_write_raw<T: AccessPhysicalMemory + VirtualAddressTranslator>(
    mem: &mut T,
    arch: Architecture,
    dtb: Address,
    addr: Address,
    data: &[u8],
) -> Result<()> {
    for (vaddr, chunk) in PageChunks::create_from(data, addr, arch.page_size()) {
        if let Ok(paddr) = mem.virt_to_phys(arch, dtb, vaddr) {
            mem.phys_write_raw(paddr, chunk)?;
        }
    }

    Ok(())
}

#[allow(unused)]
pub fn virt_page_info<T: AccessPhysicalMemory + VirtualAddressTranslator>(
    mem: &mut T,
    arch: Architecture,
    dtb: Address,
    addr: Address,
) -> Result<Page> {
    let paddr = mem.virt_to_phys(arch, dtb, addr)?;
    Ok(paddr
        .page
        .ok_or_else(|| Error::new("page info not found"))?)
}
