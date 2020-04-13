use crate::error::Result;

use crate::address::Address;
use crate::mem::AccessPhysicalMemory;

pub fn vtop<T: AccessPhysicalMemory>(
    _mem: &mut T,
    _dtb: Address,
    _addr: Address,
) -> Result<Address> {
    println!("x86_vtop() not implemented yet");
    Ok(Address::from(0))
}
