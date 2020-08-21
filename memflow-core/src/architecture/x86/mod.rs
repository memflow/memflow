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
use std::ptr;

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

fn underlying_spec(arch: &dyn Architecture) -> Option<&'static ArchWithMMU> {
    if ptr::eq(arch, x64::ARCH) {
        Some(&x64::ARCH_SPEC)
    } else if ptr::eq(arch, x32::ARCH) {
        Some(&x32::ARCH_SPEC)
    } else if ptr::eq(arch, x32_pae::ARCH) {
        Some(&x32_pae::ARCH_SPEC)
    } else {
        None
    }
}

pub fn new_translator(dtb: Address, arch: &dyn Architecture) -> Result<impl AddressTranslator> {
    let spec = underlying_spec(arch).ok_or(Error::InvalidArchitecture)?;
    Ok(X86AddressTranslator::new(spec, dtb))
}

pub fn is_x86_arch(arch: &dyn Architecture) -> bool {
    underlying_spec(arch).is_some()
}
