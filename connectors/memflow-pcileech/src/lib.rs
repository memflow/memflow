mod fpga;
mod ft60x;

use memflow_core::*;

#[allow(unused)]
pub struct Memory {
    device: fpga::Device,
}

impl Memory {
    pub fn new() -> Result<Self> {
        let mut device = fpga::Device::new()?;
        device.get_version()?;
        Ok(Self { device })
    }
}

impl PhysicalMemory for Memory {
    fn phys_read_raw_list(&mut self, _data: &mut [PhysicalReadData]) -> Result<()> {
        Err(Error::Connector(
            "memflow_pcileech::phys_read_iter not implemented",
        ))
    }

    fn phys_write_raw_list(&mut self, _data: &[PhysicalWriteData]) -> Result<()> {
        Err(Error::Connector(
            "memflow_pcileech::phys_write_iter not implemented",
        ))
    }
}
