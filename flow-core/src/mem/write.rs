use crate::error::{Error, Result};

use std::mem;
use std::ptr::copy_nonoverlapping;

use byteorder::{BigEndian, ByteOrder, LittleEndian};

use crate::address::{Address, Length};
use crate::arch::{GetArchitecture, Architecture};

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
