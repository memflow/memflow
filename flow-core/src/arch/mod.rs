pub mod x64;
pub mod x86;
pub mod x86_pae;

use crate::error::{Error, Result};
use std::convert::TryFrom;

use crate::address::{Address, Length};
use crate::mem::{AccessPhysicalMemory, PageType};

/// ByteOrder definitions
///
/// Identifies the byte order of a architecture
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum ByteOrder {
    LittleEndian,
    BigEndian,
}

/// Architecture definitions
///
/// Describes a architecture with properties
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Architecture {
    Null,
    X64,
    X86Pae,
    X86,
}

impl TryFrom<u8> for Architecture {
    type Error = Error;

    fn try_from(value: u8) -> Result<Self> {
        match value {
            0 => Ok(Architecture::Null),
            1 => Ok(Architecture::X64),
            2 => Ok(Architecture::X86Pae),
            3 => Ok(Architecture::X86),
            _ => Err(Error::new("Invalid Architecture value")),
        }
    }
}

pub struct Page {
    pub page_type: PageType,
    // TODO: others...
}

pub struct PhysicalTranslation {
    pub address: Address,
    pub page: Page,
}

#[allow(dead_code)]
impl Architecture {
    pub fn as_u8(self) -> u8 {
        match self {
            Architecture::Null => 0,
            Architecture::X64 => 1,
            Architecture::X86Pae => 2,
            Architecture::X86 => 3,
        }
    }

    pub fn bits(self) -> u8 {
        match self {
            Architecture::Null => x64::bits(),
            Architecture::X64 => x64::bits(),
            Architecture::X86Pae => x86_pae::bits(),
            Architecture::X86 => x86::bits(),
        }
    }

    pub fn byte_order(self) -> ByteOrder {
        match self {
            Architecture::Null => x64::byte_order(),
            Architecture::X64 => x64::byte_order(),
            Architecture::X86Pae => x86_pae::byte_order(),
            Architecture::X86 => x86::byte_order(),
        }
    }

    pub fn page_size(self) -> Length {
        match self {
            Architecture::Null => x64::page_size(),
            Architecture::X64 => x64::page_size(),
            Architecture::X86Pae => x86_pae::page_size(),
            Architecture::X86 => x86::page_size(),
        }
    }

    pub fn len_addr(self) -> Length {
        match self {
            Architecture::Null => x64::len_addr(),
            Architecture::X64 => x64::len_addr(),
            Architecture::X86Pae => x86_pae::len_addr(),
            Architecture::X86 => x86::len_addr(),
        }
    }

    pub fn virt_to_phys<T: AccessPhysicalMemory>(
        self,
        mem: &mut T,
        dtb: Address,
        addr: Address,
    ) -> Result<PhysicalTranslation> {
        match self {
            Architecture::Null => Ok(PhysicalTranslation {
                address: addr,
                page: Page {
                    page_type: PageType::UNKNOWN,
                },
            }),
            Architecture::X64 => x64::virt_to_phys(mem, dtb, addr),
            Architecture::X86Pae => x86_pae::virt_to_phys(mem, dtb, addr),
            Architecture::X86 => x86::virt_to_phys(mem, dtb, addr),
        }
    }
}
