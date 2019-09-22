use std::io::{Error, ErrorKind, Result};

use flow_core::mem::PhysicalMemory;

pub fn find<T: PhysicalMemory>(mem: &mut T) -> Result<()> {
    find_x64(mem)
}

pub fn find_x64<T: PhysicalMemory>(mem: &mut T) -> Result<()> {
    Ok(())
}