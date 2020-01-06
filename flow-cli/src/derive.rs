use flow_core::*;
use flow_derive::*;

#[derive(VirtualRead, VirtualWrite)]
pub struct VirtualReadWriteDerive {}

impl PhysicalRead for VirtualReadWriteDerive {
    fn phys_read(&mut self, _addr: Address, _len: Length) -> Result<Vec<u8>> {
        Err(Error::new("not implemented"))
    }
}

impl PhysicalWrite for VirtualReadWriteDerive {
    fn phys_write(&mut self, _addr: Address, _data: &[u8]) -> Result<Length> {
        Err(Error::new("not implemented"))
    }
}
