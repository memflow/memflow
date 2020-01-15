use crate::error::Result;

use crate::address::Address;
use crate::mem::PhysicalMemoryTrait;

pub fn vtop<T: PhysicalMemoryTrait>(
    _mem: &mut T,
    _dtb: Address,
    _addr: Address,
) -> Result<Address> {
    println!("x86_vtop() not implemented yet");
    Ok(Address::from(0))
}
