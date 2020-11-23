use std::prelude::v1::*;

use crate::error::Result;
use crate::mem::phys_mem::{
    PhysicalMemory, PhysicalReadData, PhysicalReadIterator, PhysicalWriteData,
    PhysicalWriteIterator,
};
use crate::types::PhysicalAddress;

use dataview::Pod;

pub struct PhysicalMemoryBatcher<'a, T: PhysicalMemory> {
    pmem: &'a mut T,
    read_list: Vec<PhysicalReadData<'a>>,
    write_list: Vec<PhysicalWriteData<'a>>,
}

impl<'a, T: PhysicalMemory> PhysicalMemoryBatcher<'a, T> {
    pub fn new(pmem: &'a mut T) -> Self {
        Self {
            pmem,
            read_list: vec![],
            write_list: vec![],
        }
    }

    pub fn read_prealloc(&mut self, capacity: usize) -> &mut Self {
        self.read_list.reserve(capacity);
        self
    }

    pub fn commit_rw(&mut self) -> Result<()> {
        if !self.read_list.is_empty() {
            self.pmem.phys_read_raw_list(&mut self.read_list)?;
            self.read_list.clear();
        }

        if !self.write_list.is_empty() {
            self.pmem.phys_write_raw_list(&self.write_list)?;
            self.write_list.clear();
        }

        Ok(())
    }

    #[inline]
    pub fn read_raw_iter<VI: PhysicalReadIterator<'a>>(&mut self, iter: VI) -> &mut Self {
        self.read_list.extend(iter);
        self
    }

    #[inline]
    pub fn write_raw_iter<VI: PhysicalWriteIterator<'a>>(&mut self, iter: VI) -> &mut Self {
        self.write_list.extend(iter);
        self
    }

    // read helpers
    #[inline]
    pub fn read_raw_into<'b: 'a>(&mut self, addr: PhysicalAddress, out: &'b mut [u8]) -> &mut Self {
        self.read_raw_iter(Some(PhysicalReadData(addr, out)).into_iter())
    }

    #[inline]
    pub fn read_into<'b: 'a, F: Pod + ?Sized>(
        &mut self,
        addr: PhysicalAddress,
        out: &'b mut F,
    ) -> &mut Self {
        self.read_raw_into(addr, out.as_bytes_mut())
    }

    // write helpers
    #[inline]
    pub fn write_raw_into<'b: 'a>(&mut self, addr: PhysicalAddress, out: &'b [u8]) -> &mut Self {
        self.write_raw_iter(Some(PhysicalWriteData(addr, out)).into_iter())
    }

    #[inline]
    pub fn write_into<'b: 'a, F: Pod + ?Sized>(
        &mut self,
        addr: PhysicalAddress,
        out: &'b F,
    ) -> &mut Self {
        self.write_raw_into(addr, out.as_bytes())
    }
}

impl<'a, T: PhysicalMemory> Drop for PhysicalMemoryBatcher<'a, T> {
    fn drop(&mut self) {
        let _ = self.commit_rw();
    }
}
