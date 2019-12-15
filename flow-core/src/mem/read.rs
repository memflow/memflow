use crate::error::Result;

use crate::address::{Address, Length};
use crate::arch::Architecture;

pub trait PhysicalRead {
    fn phys_read(&mut self, addr: Address, len: Length) -> Result<Vec<u8>>;
}

pub trait VirtualRead {
    fn virt_read(
        &mut self,
        arch: Architecture,
        dtb: Address,
        addr: Address,
        len: Length,
    ) -> Result<Vec<u8>>; // TODO: return [u8] ?
}
