pub mod x64;
pub mod x86_pae;
pub mod x86;

use crate::address::Length;

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum ByteOrder {
    LittleEndian,
    BigEndian,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum InstructionSet {
    X64,
    X86Pae,
    X86
}

macro_rules! match_instruction_set {
    ($value:expr, $func:ident) => (
        match $value {
            InstructionSet::X64 => x64::$func(),
            InstructionSet::X86Pae => x86_pae::$func(),
            InstructionSet::X86 => x86::$func(),
        }
    )
}

// TODO: change this to operate on enum variants directly
#[allow(dead_code)]
impl InstructionSet {
    fn byte_order(&self) -> ByteOrder {
        match_instruction_set!(self, byte_order)
    }

    fn len_addr(&self) -> Length {
        match_instruction_set!(self, len_addr)
    }

    fn len_u64(&self) -> Length {
        match_instruction_set!(self, len_u64)
    }

    fn len_u32(&self) -> Length {
        match_instruction_set!(self, len_u32)
    }
}

#[derive(Debug, Clone)]
pub struct Architecture {
    pub instruction_set: InstructionSet,
}

impl From<InstructionSet> for Architecture {
    fn from(item: InstructionSet) -> Self {
        Architecture{
            instruction_set: item,
        }
    }
}
