use flow_core::*;
use flow_derive::*;

#[derive(VirtualMemoryTrait)]
pub struct VirtualReadWriteDerive {}

impl PhysicalMemoryTrait for VirtualReadWriteDerive {
    fn phys_read(&mut self, _addr: Address, _out: &mut [u8]) -> Result<()> {
        Err(Error::new("not implemented"))
    }

    fn phys_write(&mut self, _addr: Address, _data: &[u8]) -> Result<()> {
        Err(Error::new("not implemented"))
    }
}
