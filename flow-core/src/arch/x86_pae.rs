use crate::error::Result;

use crate::address::{Address, Length};
use crate::arch::ByteOrder;

use crate::mem::{AccessPhysicalMemory, PageType};

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

pub fn vtop<T: AccessPhysicalMemory>(
    _mem: &mut T,
    _dtb: Address,
    _addr: Address,
) -> Result<(Address, PageType)> {
    println!("x86_pae::vtop() not implemented yet");
    Ok((Address::from(0), PageType::NONE))
}
