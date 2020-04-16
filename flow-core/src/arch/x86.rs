use crate::error::Result;

use crate::address::{Address, Length};
use crate::arch::ByteOrder;

use crate::mem::AccessPhysicalMemory;

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
) -> Result<Address> {
    println!("x86::vtop() not implemented yet");
    Ok(Address::from(0))
}
