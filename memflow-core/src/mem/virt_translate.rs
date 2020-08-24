use std::prelude::v1::*;

pub mod direct_translate;
use crate::iter::SplitAtIndex;
pub use direct_translate::DirectTranslate;

#[cfg(test)]
mod tests;

use crate::error::{Error, Result};

use crate::mem::PhysicalMemory;
use crate::types::{Address, PhysicalAddress};

use crate::architecture::ScopedVirtualTranslate;

pub trait VirtualTranslate
where
    Self: Send,
{
    fn virt_to_phys_iter<T, B, D, VI, VO, FO>(
        &mut self,
        phys_mem: &mut T,
        translator: &D,
        addrs: VI,
        out: &mut VO,
        out_fail: &mut FO,
    ) where
        T: PhysicalMemory + ?Sized,
        B: SplitAtIndex,
        D: ScopedVirtualTranslate,
        VI: Iterator<Item = (Address, B)>,
        VO: Extend<(PhysicalAddress, B)>,
        FO: Extend<(Error, Address, B)>;

    // helpers
    fn virt_to_phys<T: PhysicalMemory + ?Sized, D: ScopedVirtualTranslate>(
        &mut self,
        phys_mem: &mut T,
        translator: &D,
        vaddr: Address,
    ) -> Result<PhysicalAddress> {
        let mut vec = vec![]; //Vec::new_in(&arena);
        let mut vec_fail = vec![]; //BumpVec::new_in(&arena);
        self.virt_to_phys_iter(
            phys_mem,
            translator,
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
    fn virt_to_phys_iter<U, B, D, VI, VO, FO>(
        &mut self,
        phys_mem: &mut U,
        translator: &D,
        addrs: VI,
        out: &mut VO,
        out_fail: &mut FO,
    ) where
        U: PhysicalMemory + ?Sized,
        B: SplitAtIndex,
        D: ScopedVirtualTranslate,
        VI: Iterator<Item = (Address, B)>,
        VO: Extend<(PhysicalAddress, B)>,
        FO: Extend<(Error, Address, B)>,
    {
        (**self).virt_to_phys_iter(phys_mem, translator, addrs, out, out_fail)
    }
}
