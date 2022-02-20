pub mod x32;
pub mod x32_pae;
pub mod x64;

use super::{Architecture, ArchitectureIdent, ArchitectureObj, Endianess};

use crate::mem::virt_translate::{
    mmu::ArchMmuSpec, VirtualTranslate3, VtopFailureCallback, VtopOutputCallback,
};

use crate::error::{Error, ErrorKind, ErrorOrigin, Result};
use crate::iter::SplitAtIndex;
use crate::mem::PhysicalMemory;
use crate::types::{umem, Address};
use cglue::tuple::*;

use std::ptr;

pub struct X86Architecture {
    /// Defines how many bits does the native word size have
    bits: u8,
    /// Defines the underlying MMU used for address translation
    mmu: ArchMmuSpec,
}

impl Architecture for X86Architecture {
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
        ArchitectureIdent::X86(
            self.bits,
            ptr::eq(self as *const _, &x32_pae::ARCH_SPEC as *const _),
        )
    }
}

#[derive(Clone, Copy)]
pub struct X86VirtualTranslate {
    arch: &'static X86Architecture,
    dtb: Address,
}

impl X86VirtualTranslate {
    pub fn new(arch: &'static X86Architecture, dtb: Address) -> Self {
        Self { arch, dtb }
    }
}

impl VirtualTranslate3 for X86VirtualTranslate {
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

    fn translation_table_id(&self, _address: Address) -> umem {
        self.dtb.to_umem().overflowing_shr(12).0
    }

    fn arch(&self) -> ArchitectureObj {
        self.arch
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

pub fn new_translator(dtb: Address, arch: ArchitectureObj) -> Result<X86VirtualTranslate> {
    let arch =
        underlying_arch(arch).ok_or(Error(ErrorOrigin::Mmu, ErrorKind::InvalidArchitecture))?;
    Ok(X86VirtualTranslate::new(arch, dtb))
}

pub fn is_x86_arch(arch: ArchitectureObj) -> bool {
    underlying_arch(arch).is_some()
}
