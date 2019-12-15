use crate::error::Result;

use crate::address::{Address, Length};
use crate::arch::Architecture;

pub trait PhysicalWrite {
    fn phys_write(&mut self, addr: Address, data: &[u8]) -> Result<Length>;
}

pub trait VirtualWrite {
    fn virt_write(
        &mut self,
        arch: Architecture,
        dtb: Address,
        addr: Address,
        data: &[u8],
    ) -> Result<Length>;
}
