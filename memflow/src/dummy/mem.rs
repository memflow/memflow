use crate::connector::MappedPhysicalMemory;
use crate::error::Result;
use crate::mem::{
    MemoryMap, PhysicalMemory, PhysicalMemoryMetadata, PhysicalReadData, PhysicalWriteData,
};

use std::sync::Arc;

pub struct DummyMemory {
    pub(crate) buf: Arc<Box<[u8]>>,
    pub(crate) mem: MappedPhysicalMemory<&'static mut [u8], MemoryMap<&'static mut [u8]>>,
}

impl DummyMemory {
    pub fn new(size: usize) -> Self {
        let buf = Arc::new(vec![0_u8; size].into_boxed_slice());

        let mut map = MemoryMap::new();
        map.push_range(0.into(), buf.len().into(), (buf.as_ptr() as u64).into());

        let buf_mem = unsafe { MappedPhysicalMemory::from_addrmap_mut(map) };

        Self { buf, mem: buf_mem }
    }
}

impl Clone for DummyMemory {
    fn clone(&self) -> Self {
        let mut map = MemoryMap::new();
        map.push_range(
            0.into(),
            self.buf.len().into(),
            (self.buf.as_ptr() as u64).into(),
        );

        let mem = unsafe { MappedPhysicalMemory::from_addrmap_mut(map) };

        Self {
            buf: self.buf.clone(),
            mem,
        }
    }
}

impl PhysicalMemory for DummyMemory {
    fn phys_read_raw_list(&mut self, data: &mut [PhysicalReadData]) -> Result<()> {
        self.mem.phys_read_raw_list(data)
    }

    fn phys_write_raw_list(&mut self, data: &[PhysicalWriteData]) -> Result<()> {
        self.mem.phys_write_raw_list(data)
    }

    fn metadata(&self) -> PhysicalMemoryMetadata {
        self.mem.metadata()
    }
}
