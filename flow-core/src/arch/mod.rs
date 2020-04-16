pub mod x64;
pub mod x86;
pub mod x86_pae;

use crate::error::{Error, Result};
use std::convert::TryFrom;

use crate::address::{Address, Length};
use crate::mem::AccessPhysicalMemory;

/// ByteOrder definitions
///
/// Identifies the byte order of a architecture
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum ByteOrder {
    LittleEndian,
    BigEndian,
}

/// InstructionSet definitions
///
/// Identifies a instruction set with properties
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum InstructionSet {
    Null,
    X64,
    X86Pae,
    X86,
}

// TODO: change this to operate on enum variants directly
macro_rules! match_instruction_set {
    ($value:expr, $func:ident) => {
        match $value {
            InstructionSet::Null => x64::$func(), // TODO: zero everything out
            InstructionSet::X64 => x64::$func(),
            InstructionSet::X86Pae => x86_pae::$func(),
            InstructionSet::X86 => x86::$func(),
        }
    };
}

impl TryFrom<u8> for InstructionSet {
    type Error = Error;

    fn try_from(value: u8) -> Result<Self> {
        match value {
            0 => Ok(InstructionSet::Null),
            1 => Ok(InstructionSet::X64),
            2 => Ok(InstructionSet::X86Pae),
            3 => Ok(InstructionSet::X86),
            _ => Err(Error::new("Invalid InstructionSet value")),
        }
    }
}

#[allow(dead_code)]
impl InstructionSet {
    pub fn as_u8(self) -> u8 {
        match self {
            InstructionSet::Null => 0,
            InstructionSet::X64 => 1,
            InstructionSet::X86Pae => 2,
            InstructionSet::X86 => 3,
        }
    }

    pub fn byte_order(self) -> ByteOrder {
        match_instruction_set!(self, byte_order)
    }

    pub fn page_size(self) -> Length {
        match_instruction_set!(self, page_size)
    }

    pub fn len_addr(self) -> Length {
        match_instruction_set!(self, len_addr)
    }

    pub fn vtop<T: AccessPhysicalMemory>(
        self,
        mem: &mut T,
        dtb: Address,
        addr: Address,
    ) -> Result<Address> {
        match self {
            InstructionSet::Null => Ok(addr),
            InstructionSet::X64 => x64::vtop(mem, dtb, addr),
            InstructionSet::X86Pae => x86_pae::vtop(mem, dtb, addr),
            InstructionSet::X86 => x86::vtop(mem, dtb, addr),
        }
    }
}

// TODO: should architecture and ins set really be split up?

/// Architecture definition
///
/// Defines a target systems architecture
#[derive(Debug, Copy, Clone)]
pub struct Architecture {
    pub instruction_set: InstructionSet,
}

impl From<InstructionSet> for Architecture {
    fn from(item: InstructionSet) -> Self {
        Architecture {
            instruction_set: item,
        }
    }
}
