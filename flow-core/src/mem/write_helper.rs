use crate::error::{Error, Result};

use std::mem;
use std::ptr::copy_nonoverlapping;

use byteorder::{BigEndian, ByteOrder, LittleEndian};

use crate::address::{Address, Length};
use crate::arch::{GetArchitecture, Architecture};

// TODO: add more helper funcs

pub trait VirtualWriteHelper {
    fn virt_read(&mut self, addr: Address, len: Length) -> Result<Vec<u8>>;
}
