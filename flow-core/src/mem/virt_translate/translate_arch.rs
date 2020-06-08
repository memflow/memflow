use super::VirtualTranslate;
use crate::architecture::Architecture;
pub use crate::architecture::TranslateData;
use crate::error::Result;
use crate::mem::{
    virt_mem::{VirtualReadIterator, VirtualWriteIterator},
    PhysicalMemory,
};
use crate::types::{Address, Page, PhysicalAddress};

/*
The `TranslateArch` struct provides a default implementation for `VirtualTranslate` for physical memory.
*/
#[derive(Debug, Clone)]
pub struct TranslateArch {
    sys_arch: Architecture,
}

impl TranslateArch {
    pub fn new(sys_arch: Architecture) -> Self {
        Self { sys_arch }
    }
}

impl VirtualTranslate for TranslateArch {
    fn virt_to_phys_iter<T, B, VI, OV>(
        &mut self,
        phys_mem: &mut T,
        dtb: Address,
        addrs: VI,
        out: &mut OV,
    ) where
        T: PhysicalMemory + ?Sized,
        B: TranslateData,
        VI: Iterator<Item = (Address, B)>,
        OV: Extend<(Result<PhysicalAddress>, Address, B)>,
    {
        self.sys_arch.virt_to_phys_iter(phys_mem, dtb, addrs, out)
    }
}

pub fn virt_read_raw_iter<
    'a,
    T: PhysicalMemory + ?Sized,
    U: VirtualTranslate + ?Sized,
    VI: VirtualReadIterator<'a>,
>(
    phys_mem: &mut T,
    vat: &mut U,
    dtb: Address,
    iter: VI,
) -> Result<()> {
    //30% perf hit on dummy!!! FIXME!!!
    let mut translation = Vec::with_capacity(iter.size_hint().0);
    vat.virt_to_phys_iter(phys_mem, dtb, iter, &mut translation);

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
    U: VirtualTranslate + ?Sized,
    VI: VirtualWriteIterator<'a>,
>(
    phys_mem: &mut T,
    vat: &mut U,
    dtb: Address,
    iter: VI,
) -> Result<()> {
    //30% perf hit on dummy!!! FIXME!!!
    let mut translation = Vec::with_capacity(iter.size_hint().0);
    vat.virt_to_phys_iter(phys_mem, dtb, iter, &mut translation);

    let iter = translation.into_iter().filter_map(|(paddr, _, out)| {
        if let Ok(paddr) = paddr {
            Some((paddr, out))
        } else {
            None
        }
    });

    phys_mem.phys_write_iter(iter)
}

pub fn virt_page_info<T: PhysicalMemory + ?Sized, U: VirtualTranslate + ?Sized>(
    phys_mem: &mut T,
    vat: &mut U,
    dtb: Address,
    addr: Address,
) -> Result<Page> {
    let paddr = vat.virt_to_phys(phys_mem, dtb, addr)?;
    Ok(paddr.containing_page())
}
