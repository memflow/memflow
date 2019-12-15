use crate::error::{Error, Result};

use crate::address::Address;
use crate::mem::PhysicalRead;

pub fn vtop<T: PhysicalRead>(_mem: &mut T, _dtb: Address, _addr: Address) -> Result<Address> {
    println!("x86_vtop() not implemented yet");
    Ok(Address::from(0))
}
