use flow_core::*;
use flow_derive::*;

// TODO: open usb device
#[derive(AccessVirtualMemoryRaw, VirtualAddressTranslatorRaw)]
pub struct Memory {}

impl Memory {
    pub fn new() -> Result<Self> {
        Ok(Self {})
    }
}

impl AccessPhysicalMemory for Memory {
    fn phys_read_iter<'a, T: PhysicalReadIterator<'a>>(&mut self, _: T) -> Result<()> {
        Err(Error::new(
            "flow-pcileech::phys_read_iter not implemented",
        ))
    }

    fn phys_write_iter<'a, T: PhysicalWriteIterator<'a>>(&mut self, _: T) -> Result<()> {
        Err(Error::new(
            "flow-pcileech::phys_write_iter not implemented",
        ))
    }
}
