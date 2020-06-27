mod fpga;
mod ft60x;

use flow_core::*;

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
    fn phys_read_iter<'a, PI: PhysicalReadIterator<'a>>(&'a mut self, mut _iter: PI) -> Result<()> {
        Err(Error::Connector(
            "flow-pcileech::phys_read_iter not implemented",
        ))
    }

    fn phys_write_iter<'a, PI: PhysicalWriteIterator<'a>>(
        &'a mut self,
        mut _iter: PI,
    ) -> Result<()> {
        Err(Error::Connector(
            "flow-pcileech::phys_write_iter not implemented",
        ))
    }
}
