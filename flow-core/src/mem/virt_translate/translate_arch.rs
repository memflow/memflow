use super::VirtualTranslate;
use crate::architecture::Architecture;
pub use crate::architecture::TranslateData;
use crate::error::Result;
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
            arena: Bump::new(),
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
        B: TranslateData,
        VI: Iterator<Item = (Address, B)>,
        OV: Extend<(Result<PhysicalAddress>, Address, B)>,
    {
        self.arena.reset();
        self.sys_arch
            .virt_to_phys_iter(phys_mem, dtb, addrs, out, &self.arena)
    }
}
