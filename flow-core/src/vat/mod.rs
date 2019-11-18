//#[cfg(feature = "x64")]
pub mod x64;

//#[cfg(feature = "x86_pae")]
pub mod x86_pae;

//#[cfg(feature = "x86")]
pub mod x86;

pub mod va;
pub use va::VatImpl;

// TODO: replace by custom error results
use std::io::Result;

use crate::address::Address;
use crate::arch::{Architecture, InstructionSet};
use crate::mem::PhysicalRead;

pub trait VirtualAddressTranslation {
    fn vtop(&mut self, arch: Architecture, dtb: Address, addr: Address) -> Result<Address>;
}

impl<T: PhysicalRead> VirtualAddressTranslation for T {
    // virtual -> physical
    fn vtop(&mut self, arch: Architecture, dtb: Address, addr: Address) -> Result<Address> {
        match arch.instruction_set {
            InstructionSet::X64 => x64::vtop(self, dtb, addr),
            InstructionSet::X86Pae => x86_pae::vtop(self, dtb, addr),
            InstructionSet::X86 => x86::vtop(self, dtb, addr),
        }
    }
}
