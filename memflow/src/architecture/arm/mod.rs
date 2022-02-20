pub mod aarch64;

use super::{Architecture, ArchitectureIdent, ArchitectureObj, Endianess};

use crate::mem::virt_translate::{
    mmu::{
        translate_data::{TranslateDataVec, TranslationChunk},
        ArchMmuSpec, MmuTranslationBase,
    },
    VirtualTranslate3, VtopFailureCallback, VtopOutputCallback,
};

use crate::error::{Error, ErrorKind, ErrorOrigin, Result};
use crate::iter::SplitAtIndex;
use crate::mem::PhysicalMemory;
use crate::types::{size, umem, Address};
use cglue::tuple::*;

pub struct ArmArchitecture {
    /// Defines how many bits does the native word size have
    bits: u8,
    /// Defines the underlying MMU used for address translation
    mmu: ArchMmuSpec,
}

impl Architecture for ArmArchitecture {
    fn bits(&self) -> u8 {
        self.bits
    }

    fn endianess(&self) -> Endianess {
        self.mmu.def.endianess
    }

    fn page_size(&self) -> usize {
        self.mmu.page_size_level(1) as usize
    }

    fn size_addr(&self) -> usize {
        self.mmu.def.addr_size.into()
    }

    fn address_space_bits(&self) -> u8 {
        self.mmu.def.address_space_bits
    }

    fn ident(&self) -> ArchitectureIdent {
        ArchitectureIdent::AArch64(size::kb(4))
    }
}

// TODO: Add granularity
#[derive(Clone, Copy)]
pub struct ArmVirtualTranslate {
    arch: &'static ArmArchitecture,
    dtb: ArmPageTableBase,
}

impl ArmVirtualTranslate {
    pub fn new(arch: &'static ArmArchitecture, dtb1: Address, dtb2: Address) -> Self {
        Self {
            arch,
            dtb: ArmPageTableBase(dtb1, dtb2),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct ArmPageTableBase(Address, Address);

impl MmuTranslationBase for ArmPageTableBase {
    fn get_pt_by_virt_addr(&self, addr: Address) -> Address {
        //TODO: handle for Arm 32
        if (addr.to_umem().to_be() & 1) == 1 {
            self.1
        } else {
            self.0
        }
    }

    fn get_pt_by_index(&self, idx: usize) -> (Address, usize) {
        if idx < 256 {
            (self.0, idx)
        } else {
            (self.1, idx)
        }
    }

    fn pt_count(&self) -> usize {
        2
    }

    fn virt_addr_filter<B>(
        &self,
        spec: &ArchMmuSpec,
        addr: CTup3<Address, Address, B>,
        work_group: (&mut TranslationChunk<Self>, &mut TranslateDataVec<B>),
        out_fail: &mut VtopFailureCallback<B>,
    ) where
        B: SplitAtIndex,
    {
        spec.virt_addr_filter(addr, work_group, out_fail);
    }
}

impl VirtualTranslate3 for ArmVirtualTranslate {
    fn virt_to_phys_iter<
        T: PhysicalMemory + ?Sized,
        B: SplitAtIndex,
        VI: Iterator<Item = CTup3<Address, Address, B>>,
    >(
        &self,
        mem: &mut T,
        addrs: VI,
        out: &mut VtopOutputCallback<B>,
        out_fail: &mut VtopFailureCallback<B>,
        tmp_buf: &mut [std::mem::MaybeUninit<u8>],
    ) {
        self.arch
            .mmu
            .virt_to_phys_iter(mem, self.dtb, addrs, out, out_fail, tmp_buf)
    }

    fn translation_table_id(&self, address: Address) -> umem {
        self.dtb
            .get_pt_by_virt_addr(address)
            .to_umem()
            .overflowing_shr(11)
            .0
    }

    fn arch(&self) -> ArchitectureObj {
        self.arch
    }
}

// This lint doesn't make any sense in our usecase, since we nevel leak ARCH_SPECs, and ARCH is
// a static trait object with a consistent address.
fn underlying_arch(arch: ArchitectureObj) -> Option<&'static ArmArchitecture> {
    if arch == aarch64::ARCH {
        Some(&aarch64::ARCH_SPEC)
    } else {
        None
    }
}

pub fn new_translator(
    dtb1: Address,
    dtb2: Address,
    arch: ArchitectureObj,
) -> Result<impl VirtualTranslate3> {
    let arch =
        underlying_arch(arch).ok_or(Error(ErrorOrigin::Mmu, ErrorKind::InvalidArchitecture))?;
    Ok(ArmVirtualTranslate::new(arch, dtb1, dtb2))
}

pub fn new_translator_nonsplit(
    dtb: Address,
    arch: ArchitectureObj,
) -> Result<impl VirtualTranslate3> {
    // TODO: Handle 32 bit arm
    new_translator(dtb, dtb + size::kb(2), arch)
}

pub fn is_arm_arch(arch: ArchitectureObj) -> bool {
    underlying_arch(arch).is_some()
}
