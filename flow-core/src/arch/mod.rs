pub mod x64;
pub mod x86;
pub mod x86_pae;

use std::convert::TryFrom;
use std::io::{Error, ErrorKind, Result};

use crate::address::Length;

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum ByteOrder {
    LittleEndian,
    BigEndian,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum InstructionSet {
    X64,
    X86Pae,
    X86,
}

// TODO: change this to operate on enum variants directly
macro_rules! match_instruction_set {
    ($value:expr, $func:ident) => {
        match $value {
            InstructionSet::X64 => x64::$func(),
            InstructionSet::X86Pae => x86_pae::$func(),
            InstructionSet::X86 => x86::$func(),
        }
    };
}

// TODO: figure out a better way for this
impl TryFrom<u8> for InstructionSet {
    type Error = std::io::Error;

    fn try_from(value: u8) -> Result<Self> {
        match value {
            1 => Ok(InstructionSet::X64),
            2 => Ok(InstructionSet::X86Pae),
            3 => Ok(InstructionSet::X86),
            _ => Err(Error::new(ErrorKind::Other, "Invalid InstructionSet value")),
        }
    }
}

#[allow(dead_code)]
impl InstructionSet {
    pub fn as_u8(self) -> u8 {
        match self {
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

    pub fn len_u64(self) -> Length {
        match_instruction_set!(self, len_u64)
    }

    pub fn len_u32(self) -> Length {
        match_instruction_set!(self, len_u32)
    }

    pub fn len_u16(self) -> Length {
        match_instruction_set!(self, len_u16)
    }

    pub fn len_u8(self) -> Length {
        match_instruction_set!(self, len_u8)
    }

    pub fn len_i64(self) -> Length {
        match_instruction_set!(self, len_i64)
    }

    pub fn len_i32(self) -> Length {
        match_instruction_set!(self, len_i32)
    }

    pub fn len_i16(self) -> Length {
        match_instruction_set!(self, len_i16)
    }

    pub fn len_i8(self) -> Length {
        match_instruction_set!(self, len_i8)
    }

    pub fn len_f32(self) -> Length {
        match_instruction_set!(self, len_f32)
    }
}

// TODO: do we gain anything from this wrap?
// TODO: do we need any other fields?
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
