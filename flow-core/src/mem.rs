// TODO: custom error + result
use std::io::Result;

use crate::address::{Address, Length};
use crate::arch::Architecture;

pub trait PhysicalRead {
    fn phys_read(&mut self, addr: Address, len: Length) -> Result<Vec<u8>>;
}

pub trait VirtualRead {
    fn virt_read(&mut self, arch: Architecture, dtb: Address, addr: Address, len: Length) -> Result<Vec<u8>>;
}

pub trait PhysicalWrite {
    fn phys_write(&mut self, addr: Address, data: &Vec<u8>) -> Result<Length>;
}

pub trait VirtualWrite {
    fn write_mem(&mut self, arch: Architecture, dtb: Address, addr: Address, data: &Vec<u8>) -> Result<Length>;
}
