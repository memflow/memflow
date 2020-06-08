pub mod translate_arch;
pub use translate_arch::{TranslateArch, TranslateData};

#[cfg(test)]
mod tests;

use crate::error::Result;

use crate::mem::PhysicalMemory;
use crate::types::{Address, PhysicalAddress};

pub trait VirtualTranslate {
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

// forward impls
impl<'a, T: VirtualTranslate> VirtualTranslate for &'a mut T {
    fn virt_to_phys_iter<U, B, VI, OV>(
        &mut self,
        phys_mem: &mut U,
        dtb: Address,
        addrs: VI,
        out: &mut OV,
    ) where
        U: PhysicalMemory + ?Sized,
        B: TranslateData,
        VI: Iterator<Item = (Address, B)>,
        OV: Extend<(Result<PhysicalAddress>, Address, B)>,
    {
        (*self).virt_to_phys_iter(phys_mem, dtb, addrs, out)
    }
}
