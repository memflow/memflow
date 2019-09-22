use std::io::Result;

// TODO: custom error + result
pub trait PhysicalMemory {
    fn read_physical_memory(&mut self, addr: u64, len: u64) -> Result<Vec<u8>>;
    fn write_physical_memory(&mut self, addr: u64, data: &Vec<u8>) -> Result<u64>;
}