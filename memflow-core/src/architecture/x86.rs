pub mod x32;
pub mod x32_pae;
pub mod x64;

use super::{
    mmu_spec::{ArchWithMMU, MMUTranslationBase},
    AddressTranslator, Architecture,
};

use super::{Bump, BumpVec};
use crate::error::{Error, Result};
use crate::iter::SplitAtIndex;
use crate::mem::PhysicalMemory;
use crate::types::{Address, PhysicalAddress};

#[derive(Clone, Copy)]
pub struct X86AddressTranslator {
    spec: &'static ArchWithMMU,
    dtb: X86PageTableBase,
}

impl X86AddressTranslator {
    pub fn new(spec: &'static ArchWithMMU, dtb: Address) -> Self {
        Self {
            spec,
            dtb: X86PageTableBase(dtb),
        }
    }
}

impl AddressTranslator for X86AddressTranslator {
    fn virt_to_phys_iter<
        T: PhysicalMemory + ?Sized,
        B: SplitAtIndex,
        VI: Iterator<Item = (Address, B)>,
        VO: Extend<(PhysicalAddress, B)>,
        FO: Extend<(Error, Address, B)>,
    >(
        &self,
        mem: &mut T,
        addrs: VI,
        out: &mut VO,
        out_fail: &mut FO,
        arena: &Bump,
    ) {
        self.spec
            .virt_to_phys_iter(mem, self.dtb, addrs, out, out_fail, arena)
    }

    fn translation_table_id(&self, address: Address) -> usize {
        self.dtb.0.as_u64().overflowing_shr(12).0 as usize
    }

    fn arch(&self) -> &dyn Architecture {
        self.spec
    }
}

#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct X86PageTableBase(Address);

impl MMUTranslationBase for X86PageTableBase {
    fn get_initial_pt(&self, _: Address) -> Address {
        self.0
    }
}
