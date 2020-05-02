use crate::error::{Error, Result};

use super::PhysicalTranslation;
use crate::address::{Address, Length};
use crate::arch::ByteOrder;

use crate::mem::AccessPhysicalMemory;

pub fn bits() -> u8 {
    32
}

pub fn byte_order() -> ByteOrder {
    ByteOrder::LittleEndian
}

pub fn page_size() -> Length {
    Length::from_kb(4)
}

pub fn len_addr() -> Length {
    Length::from(4)
}

pub fn virt_to_phys<T: AccessPhysicalMemory>(
    _mem: &mut T,
    _dtb: Address,
    _addr: Address,
) -> Result<PhysicalTranslation> {
    println!("x86::virt_to_phys() not implemented yet");
    Err(Error::new("x86::virt_to_phys() not implemented yet"))
}
