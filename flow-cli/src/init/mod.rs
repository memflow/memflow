pub mod init_bridge;
pub mod init_qemu_procfs;

// empty implementation
use flow_core::*;
use flow_derive::*;

#[derive(VirtualMemoryTrait)]
pub struct EmptyVirtualMemory {}

impl PhysicalMemoryTrait for EmptyVirtualMemory {
    fn phys_read(&mut self, _addr: Address, _out: &mut [u8]) -> Result<()> {
        Err(Error::new("phys_read not implemented"))
    }

    fn phys_write(&mut self, _addr: Address, _data: &[u8]) -> Result<()> {
        Err(Error::new("phys_write not implemented"))
    }
}
