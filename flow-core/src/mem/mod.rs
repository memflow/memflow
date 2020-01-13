pub mod read;
pub mod read_helper;
pub mod write;
pub mod write_helper;

pub use read::{PhysicalRead, VirtualRead};
pub use read_helper::{
    VirtualReadHelper, VirtualReadHelperChain, VirtualReadHelperFuncs, VirtualReader,
};
pub use write::{PhysicalWrite, VirtualWrite};
pub use write_helper::{VirtualWriteHelper, VirtualWriteHelperFuncs, VirtualWriter};

use crate::arch::Architecture;
use crate::Result;

// TypeArchitectureTrait - determines the architecture for virtual read types
pub trait TypeArchitectureTrait {
    fn type_arch(&mut self) -> Result<Architecture>;
}

// much simplified version here
/*
pub use crate::address::*;

pub trait MemoryReadWrite {
    fn read(&mut self, addr: Address, out: &[u8]) -> Result<()>;
    fn write(&mut self, addr: Address, data: &[u8]) -> Result<()>;
}

pub trait PhysicalMemory<T: MemoryReadWrite> {
}

pub trait VirtualMemory<T: MemoryReadWrite> {
    fn virt_mem(&mut self, arch: Architecture, dtb: Address) -> Result<T>;
}
*/
