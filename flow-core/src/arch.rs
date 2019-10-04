use crate::addr::Length;

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

pub trait TypeLengths {
    fn len_addr(&self) -> Length;
}

// TODO: split up implementation for each type
impl TypeLengths for InstructionSet {
    fn len_addr(&self) -> Length {
        match self {
            InstructionSet::X64 => Length::from(8),
            InstructionSet::X86Pae => Length::from(4),
            InstructionSet::X86 => Length::from(4),
        }
    }
}

// TODO: create a typeSize helper for each instruction set to convert type lengths! (extend Length!)

pub fn byte_order(ins: &InstructionSet) -> ByteOrder {
    match ins {
        InstructionSet::X64 => ByteOrder::LittleEndian,
        InstructionSet::X86Pae => ByteOrder::LittleEndian,
        InstructionSet::X86 => ByteOrder::LittleEndian,
    }
}

#[derive(Debug, Clone)]
pub struct Architecture {
    pub byte_order: ByteOrder,
    pub instruction_set: InstructionSet,
}

impl From<InstructionSet> for Architecture {
    fn from(item: InstructionSet) -> Self {
        Architecture{
            byte_order: byte_order(&item),
            instruction_set: item,
        }
    }
}
