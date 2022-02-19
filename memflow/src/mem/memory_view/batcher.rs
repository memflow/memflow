use std::prelude::v1::*;

use super::*;
use crate::dataview::Pod;
use crate::error::PartialResult;
use crate::types::Address;

pub struct MemoryViewBatcher<'a, T: MemoryView> {
    vmem: &'a mut T,
    read_list: Vec<ReadData<'a>>,
    write_list: Vec<WriteData<'a>>,
}

impl<'a, T: MemoryView> MemoryViewBatcher<'a, T> {
    pub fn new(vmem: &'a mut T) -> Self {
        Self {
            vmem,
            read_list: vec![],
            write_list: vec![],
        }
    }

    pub fn read_prealloc(&mut self, capacity: usize) -> &mut Self {
        self.read_list.reserve(capacity);
        self
    }

    pub fn commit_rw(&mut self) -> PartialResult<()> {
        if !self.read_list.is_empty() {
            self.vmem.read_raw_list(&mut self.read_list)?;
            self.read_list.clear();
        }

        if !self.write_list.is_empty() {
            self.vmem.write_raw_list(&self.write_list)?;
            self.write_list.clear();
        }

        Ok(())
    }

    pub fn read_raw_iter(&mut self, iter: impl ReadIterator<'a>) -> &mut Self {
        self.read_list.extend(iter);
        self
    }

    pub fn write_raw_iter(&mut self, iter: impl WriteIterator<'a>) -> &mut Self {
        self.write_list.extend(iter);
        self
    }

    // read helpers
    pub fn read_raw_into<'b: 'a>(&mut self, addr: Address, out: &'b mut [u8]) -> &mut Self {
        self.read_raw_iter(std::iter::once(MemData2(addr, out.into())).into_iter())
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
        self.write_raw_iter(std::iter::once(MemData2(addr, out.into())).into_iter())
    }

    pub fn write_into<'b: 'a, F: Pod + ?Sized>(&mut self, addr: Address, out: &'b F) -> &mut Self {
        self.write_raw_into(addr, out.as_bytes())
    }
}

impl<'a, T: MemoryView> Drop for MemoryViewBatcher<'a, T> {
    fn drop(&mut self) {
        let _ = self.commit_rw();
    }
}
