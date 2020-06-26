use super::VirtualTranslate;
use crate::architecture::Architecture;
use crate::error::Result;
use crate::iter::SplitAtIndex;
use crate::mem::PhysicalMemory;
use crate::types::{Address, PhysicalAddress};
use bumpalo::Bump;

/*
The `TranslateArch` struct provides a default implementation for `VirtualTranslate` for physical memory.
*/
#[derive(Debug)]
pub struct TranslateArch {
    sys_arch: Architecture,
    arena: Bump,
}

impl TranslateArch {
    pub fn new(sys_arch: Architecture) -> Self {
        Self {
            sys_arch,
            arena: Bump::with_capacity(0x4000),
        }
    }
}

impl Clone for TranslateArch {
    fn clone(&self) -> Self {
        Self::new(self.sys_arch)
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
        B: SplitAtIndex,
        VI: Iterator<Item = (Address, B)>,
        OV: Extend<(Result<PhysicalAddress>, Address, B)>,
    {
        self.arena.reset();
        self.sys_arch
            .virt_to_phys_iter(phys_mem, dtb, addrs, out, &self.arena)
    }
}
