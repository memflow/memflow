use crate::error::{Error, Result};

use std::mem;
use std::ptr::copy_nonoverlapping;

use byteorder::{BigEndian, ByteOrder, LittleEndian};

use crate::address::{Address, Length};
use crate::arch::{GetArchitecture, Architecture};

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
