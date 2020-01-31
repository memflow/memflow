use log::info;

use flow_core::*;

// TODO: open usb device
pub struct Memory {
}

#[derive(VirtualMemoryTrait)]
impl Memory {
    pub fn new() -> Result<Self> {
        Ok(Self {
        })
    }
}

impl PhysicalMemoryTrait for Memory {
    fn phys_read_raw(&mut self, addr: Address, out: &mut [u8]) -> Result<()> {
        Ok(())
    }

    fn phys_write_raw(&mut self, addr: Address, data: &[u8]) -> Result<()> {
        Ok(())
    }
}
