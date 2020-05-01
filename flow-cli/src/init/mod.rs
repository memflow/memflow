pub mod init_bridge;
pub mod init_qemu_procfs;

// empty implementation
use flow_core::*;
use flow_derive::*;

#[derive(AccessVirtualMemory)]
pub struct EmptyVirtualMemory {}

impl AccessPhysicalMemory for EmptyVirtualMemory {
    fn phys_read_raw_into(
        &mut self,
        _addr: Address,
        _page_type: PageType,
        _out: &mut [u8],
    ) -> Result<()> {
        Err(Error::new("phys_read not implemented"))
    }

    fn phys_write_raw(&mut self, _addr: Address, _page_type: PageType, _data: &[u8]) -> Result<()> {
        Err(Error::new("phys_write not implemented"))
    }
}
