pub mod x32;
pub mod x32_pae;
pub mod x64;

use super::{
    mmu_spec::{translate_data::TranslateVec, ArchMMUSpec, MMUTranslationBase},
    Architecture, ArchitectureObj, Endianess, ScopedVirtualTranslate,
};

use super::Bump;
use crate::error::{Error, Result};
use crate::iter::SplitAtIndex;
use crate::mem::PhysicalMemory;
use crate::types::{Address, PhysicalAddress};

pub struct X86Architecture {
    /// Defines how many bits does the native word size have
    bits: u8,
    /// Defines the byte order of the architecture
    endianess: Endianess,
    /// Defines the underlying MMU used for address translation
    mmu: ArchMMUSpec,
}

impl Architecture for X86Architecture {
    fn bits(&self) -> u8 {
        self.bits
    }

    fn endianess(&self) -> Endianess {
        self.endianess
    }

    fn page_size(&self) -> usize {
        self.mmu.page_size_level(1)
    }

    fn size_addr(&self) -> usize {
        self.mmu.addr_size.into()
    }

    fn address_space_bits(&self) -> u8 {
        self.mmu.address_space_bits
    }
}

#[derive(Clone, Copy)]
pub struct X86ScopedVirtualTranslate {
    arch: &'static X86Architecture,
    dtb: X86PageTableBase,
}

impl X86ScopedVirtualTranslate {
    pub fn new(arch: &'static X86Architecture, dtb: Address) -> Self {
        Self {
            arch,
            dtb: X86PageTableBase(dtb),
        }
    }
}

impl ScopedVirtualTranslate for X86ScopedVirtualTranslate {
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
        self.arch
            .mmu
            .virt_to_phys_iter(mem, self.dtb, addrs, out, out_fail, arena)
    }

    fn translation_table_id(&self, _address: Address) -> usize {
        self.dtb.0.as_u64().overflowing_shr(12).0 as usize
    }

    fn arch(&self) -> ArchitectureObj {
        self.arch
    }
}

#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct X86PageTableBase(Address);

impl MMUTranslationBase for X86PageTableBase {
    fn get_initial_pt(&self, _: Address) -> Address {
        self.0
    }

    fn get_pt_by_index(&self, _: usize) -> Address {
        self.0
    }

    fn pt_count(&self) -> usize {
        1
    }

    fn virt_addr_filter<B, O>(
        &self,
        spec: &ArchMMUSpec,
        addr: (Address, B),
        data_to_translate: &mut TranslateVec<B>,
        out_fail: &mut O,
    ) where
        B: SplitAtIndex,
        O: Extend<(Error, Address, B)>,
    {
        spec.virt_addr_filter(addr, &mut data_to_translate[0].vec, out_fail);
    }
}

// This lint doesn't make any sense in our usecase, since we nevel leak ARCH_SPECs, and ARCH is
// a static trait object with a consistent address.
fn underlying_arch(arch: ArchitectureObj) -> Option<&'static X86Architecture> {
    if arch == x64::ARCH {
        Some(&x64::ARCH_SPEC)
    } else if arch == x32::ARCH {
        Some(&x32::ARCH_SPEC)
    } else if arch == x32_pae::ARCH {
        Some(&x32_pae::ARCH_SPEC)
    } else {
        None
    }
}

pub fn new_translator(dtb: Address, arch: ArchitectureObj) -> Result<impl ScopedVirtualTranslate> {
    let arch = underlying_arch(arch).ok_or(Error::InvalidArchitecture)?;
    Ok(X86ScopedVirtualTranslate::new(arch, dtb))
}

pub fn is_x86_arch(arch: ArchitectureObj) -> bool {
    underlying_arch(arch).is_some()
}
