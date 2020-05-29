pub mod init_bridge;
pub mod init_qemu_procfs;

// empty implementation
use flow_core::*;
use flow_derive::*;

#[derive(VirtualAddressTranslatorRaw, AccessVirtualMemoryRaw)]
pub struct EmptyVirtualMemory {}

impl AccessPhysicalMemory for EmptyVirtualMemory {
    fn phys_read_iter<'a, PI: PhysicalReadIterator<'a>>(&'a mut self, _iter: PI) -> Result<()> {
        Err(Error::new("phys_read not implemented"))
    }

    fn phys_write_iter<'a, PI: PhysicalWriteIterator<'a>>(&'a mut self, _iter: PI) -> Result<()> {
        Err(Error::new("phys_read not implemented"))
    }
}
