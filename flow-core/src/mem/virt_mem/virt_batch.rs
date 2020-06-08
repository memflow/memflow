use super::{VirtualMemory, VirtualReadData, VirtualReadIterator, VirtualWriteIterator};
use crate::error::{Error, Result};
use crate::types::{Address, Page};

pub struct VirtualMemoryBatch<'a> {
    read_batch: Vec<(Address, &'a mut [u8])>,
}

impl<'a> VirtualMemoryBatch<'a> {
    pub fn new() -> Self {
        Self {
            read_batch: Vec::new(),
        }
    }

    pub fn virt_read_raw_into(&mut self, addr: Address, out: &'a mut [u8]) {
        self.read_batch.push((addr, out));
    }
}

impl<'a> IntoIterator for VirtualMemoryBatch<'a> {
    type Item = VirtualReadData<'a>;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.read_batch.into_iter()
    }
}
