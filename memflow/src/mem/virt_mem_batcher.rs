use std::prelude::v1::*;

use crate::error::PartialResult;
use crate::mem::virt_mem::{
    VirtualMemory, VirtualReadData, VirtualReadIterator, VirtualWriteData, VirtualWriteIterator,
};
use crate::types::Address;

use dataview::Pod;

pub struct VirtualMemoryBatcher<'a, T: VirtualMemory> {
    vmem: &'a mut T,
    read_list: Vec<VirtualReadData<'a>>,
    write_list: Vec<VirtualWriteData<'a>>,
}

impl<'a, T: VirtualMemory> VirtualMemoryBatcher<'a, T> {
    pub fn new(vmem: &'a mut T) -> Self {
        Self {
            vmem,
            read_list: vec![],
            write_list: vec![],
        }
    }

    pub fn commit_rw(&mut self) -> PartialResult<()> {
        if !self.read_list.is_empty() {
            self.vmem.virt_read_raw_list(&mut self.read_list)?;
            self.read_list.clear();
        }

        if !self.write_list.is_empty() {
            self.vmem.virt_write_raw_list(&self.write_list)?;
            self.write_list.clear();
        }

        Ok(())
    }

    pub fn read_raw_iter<VI: VirtualReadIterator<'a>>(&mut self, iter: VI) -> &mut Self {
        self.read_list.extend(iter);
        self
    }

    pub fn write_raw_iter<VI: VirtualWriteIterator<'a>>(&mut self, iter: VI) -> &mut Self {
        self.write_list.extend(iter);
        self
    }

    // read helpers
    pub fn read_raw_into<'b: 'a>(&mut self, addr: Address, out: &'b mut [u8]) -> &mut Self {
        self.read_raw_iter(Some(VirtualReadData(addr, out)).into_iter())
    }

    pub fn read_into<'b: 'a, F: Pod + ?Sized>(
        &mut self,
        addr: Address,
        out: &'b mut F,
    ) -> &mut Self {
        self.read_raw_into(addr, out.as_bytes_mut())
    }

    // write helpers
    pub fn write_raw_into<'b: 'a>(&mut self, addr: Address, out: &'b [u8]) -> &mut Self {
        self.write_raw_iter(Some(VirtualWriteData(addr, out)).into_iter())
    }

    pub fn write_into<'b: 'a, F: Pod + ?Sized>(&mut self, addr: Address, out: &'b F) -> &mut Self {
        self.write_raw_into(addr, out.as_bytes())
    }
}

impl<'a, T: VirtualMemory> Drop for VirtualMemoryBatcher<'a, T> {
    fn drop(&mut self) {
        let _ = self.commit_rw();
    }
}
