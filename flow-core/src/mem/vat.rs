#[cfg(test)]
mod tests;

use crate::error::Result;

use crate::architecture::Architecture;
use crate::iter::page_chunks::{PageChunks, PageChunksMut};
use crate::mem::{
    virt_mem::{VirtualReadIterator, VirtualWriteIterator},
    PhysicalMemory,
};
use crate::types::{Address, Page, PhysicalAddress};

pub trait VAT {
    fn virt_to_phys_iter<T, B, VI, OV>(
        &mut self,
        phys_mem: &mut T,
        dtb: Address,
        addrs: VI,
        out: &mut OV,
    ) where
        T: PhysicalMemory + ?Sized,
        VI: Iterator<Item = (Address, B)>,
        OV: Extend<(Result<PhysicalAddress>, Address, B)>;

    // helpers
    fn virt_to_phys<T: PhysicalMemory + ?Sized>(
        &mut self,
        phys_mem: &mut T,
        dtb: Address,
        vaddr: Address,
    ) -> Result<PhysicalAddress> {
        let mut out = Vec::with_capacity(1);
        self.virt_to_phys_iter(phys_mem, dtb, Some((vaddr, false)).into_iter(), &mut out);
        out.pop().unwrap().0
    }
}

//
//
//

// TODO: rename trait + impl
// impl
pub struct VirtualAdressTranslator {
    sys_arch: Architecture,
}

impl VirtualAdressTranslator {
    pub fn new(sys_arch: Architecture) -> Self {
        Self { sys_arch }
    }
}

impl VAT for VirtualAdressTranslator {
    fn virt_to_phys_iter<T, B, VI, OV>(
        &mut self,
        phys_mem: &mut T,
        dtb: Address,
        addrs: VI,
        out: &mut OV,
    ) where
        T: PhysicalMemory + ?Sized,
        VI: Iterator<Item = (Address, B)>,
        OV: Extend<(Result<PhysicalAddress>, Address, B)>,
    {
        self.sys_arch.virt_to_phys_iter(phys_mem, dtb, addrs, out)
    }
}

//
//
//

pub fn virt_read_raw_iter<
    'a,
    T: PhysicalMemory + ?Sized,
    U: VAT + ?Sized,
    VI: VirtualReadIterator<'a>,
>(
    phys_mem: &mut T,
    vat: &mut U,
    arch: Architecture,
    dtb: Address,
    iter: VI,
) -> Result<()> {
    //30% perf hit on dummy!!! FIXME!!!
    let mut translation = Vec::with_capacity(iter.size_hint().0);
    vat.virt_to_phys_iter(
        phys_mem,
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

    phys_mem.phys_read_iter(iter)
}

pub fn virt_write_raw_iter<
    'a,
    T: PhysicalMemory + ?Sized,
    U: VAT + ?Sized,
    VI: VirtualWriteIterator<'a>,
>(
    phys_mem: &mut T,
    vat: &mut U,
    arch: Architecture,
    dtb: Address,
    iter: VI,
) -> Result<()> {
    //30% perf hit on dummy!!! FIXME!!!
    let mut translation = Vec::with_capacity(iter.size_hint().0);
    vat.virt_to_phys_iter(
        phys_mem,
        dtb,
        iter.flat_map(|(addr, out)| PageChunks::create_from(out, addr, arch.page_size())),
        &mut translation,
    );

    let iter = translation.into_iter().filter_map(|(paddr, _, out)| {
        if let Ok(paddr) = paddr {
            Some((paddr, out))
        } else {
            None
        }
    });

    phys_mem.phys_write_iter(iter)
}

#[allow(unused)]
pub fn virt_page_info<T: PhysicalMemory + ?Sized, U: VAT + ?Sized>(
    phys_mem: &mut T,
    vat: &mut U,
    arch: Architecture,
    dtb: Address,
    addr: Address,
) -> Result<Page> {
    let paddr = vat.virt_to_phys(phys_mem, dtb, addr)?;
    Ok(paddr.containing_page())
}
