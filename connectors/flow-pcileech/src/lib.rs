use flow_core::*;
use flow_derive::*;

// TODO: open usb device
#[derive(AccessVirtualMemory)]
pub struct Memory {}

impl Memory {
    pub fn new() -> Result<Self> {
        Ok(Self {})
    }
}

impl AccessPhysicalMemory for Memory {
    fn phys_read_raw_into(
        &mut self,
        _addr: Address,
        _page_type: mem::PageType,
        _out: &mut [u8],
    ) -> Result<()> {
        Ok(())
    }

    fn phys_write_raw(
        &mut self,
        _addr: Address,
        _page_type: mem::PageType,
        _data: &[u8],
    ) -> Result<()> {
        Ok(())
    }
}
