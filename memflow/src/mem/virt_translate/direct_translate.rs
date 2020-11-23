use super::VirtualTranslate;
use crate::architecture::ScopedVirtualTranslate;
use crate::error::Error;
use crate::iter::SplitAtIndex;
use crate::mem::PhysicalMemory;
use crate::types::{size, Address, PhysicalAddress};

/*
The `DirectTranslate` struct provides a default implementation for `VirtualTranslate` for physical memory.
*/
#[derive(Debug, Default)]
pub struct DirectTranslate {
    tmp_buf: Box<[std::mem::MaybeUninit<u8>]>,
}

impl DirectTranslate {
    pub fn new() -> Self {
        Self::with_capacity(size::mb(128))
    }

    pub fn with_capacity(size: usize) -> Self {
        Self {
            tmp_buf: vec![std::mem::MaybeUninit::new(0); size].into_boxed_slice(),
        }
    }
}

impl Clone for DirectTranslate {
    fn clone(&self) -> Self {
        Self::with_capacity(self.tmp_buf.len())
    }
}

impl VirtualTranslate for DirectTranslate {
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
        FO: Extend<(Error, Address, B)>,
    {
        translator.virt_to_phys_iter(phys_mem, addrs, out, out_fail, &mut self.tmp_buf)
    }
}
