use super::VirtualTranslate;
use crate::architecture::ScopedVirtualTranslate;
use crate::error::Error;
use crate::iter::SplitAtIndex;
use crate::mem::PhysicalMemory;
use crate::types::{Address, PhysicalAddress};
use bumpalo::Bump;

/*
The `DirectTranslate` struct provides a default implementation for `VirtualTranslate` for physical memory.
*/
#[derive(Debug, Default)]
pub struct DirectTranslate {
    arena: Bump,
}

impl DirectTranslate {
    pub fn new() -> Self {
        Self {
            arena: Bump::with_capacity(0x4000),
        }
    }
}

impl Clone for DirectTranslate {
    fn clone(&self) -> Self {
        Self::new()
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
        self.arena.reset();
        translator.virt_to_phys_iter(phys_mem, addrs, out, out_fail, &self.arena)
    }
}
