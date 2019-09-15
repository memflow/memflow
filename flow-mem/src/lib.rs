use std::io::Result;

// TODO: custom error
pub trait PhysicalMemory {
    fn read_physical_memory(&mut self, addr: u64, len: u64) -> Result<Vec<u8>>;
    fn write_physical_memory(&mut self, addr: u64, data: &Vec<u8>) -> Result<u64>;
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
