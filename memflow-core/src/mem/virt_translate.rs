use std::prelude::v1::*;

pub mod translate_arch;
use crate::iter::SplitAtIndex;
pub use translate_arch::TranslateArch;

#[cfg(test)]
mod tests;

use crate::error::{Error, Result};

use crate::mem::PhysicalMemory;
use crate::types::{Address, PhysicalAddress};

pub trait VirtualTranslate
where
    Self: Send,
{
    fn virt_to_phys_iter<T, B, VI, VO, FO>(
        &mut self,
        phys_mem: &mut T,
        dtb: Address,
        addrs: VI,
        out: &mut VO,
        out_fail: &mut FO,
    ) where
        T: PhysicalMemory + ?Sized,
        B: SplitAtIndex,
        VI: Iterator<Item = (Address, B)>,
        VO: Extend<(PhysicalAddress, B)>,
        FO: Extend<(Error, Address, B)>;

    // helpers
    fn virt_to_phys<T: PhysicalMemory + ?Sized>(
        &mut self,
        phys_mem: &mut T,
        dtb: Address,
        vaddr: Address,
    ) -> Result<PhysicalAddress> {
        let mut vec = vec![]; //Vec::new_in(&arena);
        let mut vec_fail = vec![]; //BumpVec::new_in(&arena);
        self.virt_to_phys_iter(
            phys_mem,
            dtb,
            Some((vaddr, 1)).into_iter(),
            &mut vec,
            &mut vec_fail,
        );
        if let Some(ret) = vec.pop() {
            Ok(ret.0)
        } else {
            Err(vec_fail.pop().unwrap().0)
        }
    }
}

// forward impls
impl<'a, T, P> VirtualTranslate for P
where
    T: VirtualTranslate + ?Sized,
    P: std::ops::DerefMut<Target = T> + Send,
{
    #[inline]
    fn virt_to_phys_iter<U, B, VI, VO, FO>(
        &mut self,
        phys_mem: &mut U,
        dtb: Address,
        addrs: VI,
        out: &mut VO,
        out_fail: &mut FO,
    ) where
        U: PhysicalMemory + ?Sized,
        B: SplitAtIndex,
        VI: Iterator<Item = (Address, B)>,
        VO: Extend<(PhysicalAddress, B)>,
        FO: Extend<(Error, Address, B)>,
    {
        (**self).virt_to_phys_iter(phys_mem, dtb, addrs, out, out_fail)
    }
}
