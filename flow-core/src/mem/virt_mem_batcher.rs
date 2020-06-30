use std::prelude::v1::*;

use crate::types::Address;
use crate::virt_mem::{
    VirtualMemory, VirtualReadData, VirtualReadIterator, VirtualWriteData, VirtualWriteIterator,
};
use crate::Result;
use core::mem::replace;

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

    pub fn commit_rw(&mut self) -> Result<()> {
        let read_list = replace(&mut self.read_list, vec![]);

        if !read_list.is_empty() {
            self.vmem.virt_read_iter(read_list.into_iter())?;
        }

        let write_list = replace(&mut self.write_list, vec![]);

        if !write_list.is_empty() {
            self.vmem.virt_write_iter(write_list.into_iter())?;
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
        self.read_raw_iter(Some((addr, out)).into_iter())
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
        self.write_raw_iter(Some((addr, out)).into_iter())
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
