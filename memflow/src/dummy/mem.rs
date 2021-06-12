use crate::connector::MappedPhysicalMemory;
use crate::derive::connector;
use crate::error::{Error, ErrorKind, ErrorOrigin, Result};
use crate::mem::{
    MemoryMap, PhysicalMemory, PhysicalMemoryMetadata, PhysicalReadData, PhysicalWriteData,
};
use crate::plugins::*;

use crate::plugins::Args;
use crate::types::{size, Address};

use cglue::*;
use std::sync::Arc;

cglue_impl_group!(DummyMemory, ConnectorInstance, {});

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

    fn set_mem_map(&mut self, mem_map: MemoryMap<(Address, usize)>) {
        self.mem.set_mem_map(mem_map)
    }
}

pub fn parse_size(args: &Args) -> Result<usize> {
    let (size, size_mul) = {
        let size = args.get("size").unwrap_or("2m");

        let mul_arr = &[
            (size::kb(1), ["kb", "k"]),
            (size::mb(1), ["mb", "m"]),
            (size::gb(1), ["gb", "g"]),
        ];

        mul_arr
            .iter()
            .flat_map(|(m, e)| e.iter().map(move |e| (*m, e)))
            .filter_map(|(m, e)| {
                if size.to_lowercase().ends_with(e) {
                    Some((size.trim_end_matches(e), m))
                } else {
                    None
                }
            })
            .next()
            .ok_or(Error(
                ErrorOrigin::Connector,
                ErrorKind::InvalidMemorySizeUnit,
            ))?
    };

    let size = usize::from_str_radix(size, 16)
        .map_err(|_| Error(ErrorOrigin::Connector, ErrorKind::InvalidMemorySize))?;

    Ok(size * size_mul)
}

#[connector(name = "dummy", import_prefix = "crate")]
pub fn create_connector(args: &Args) -> Result<impl PhysicalMemory + Clone> {
    let size = parse_size(args)?;
    Ok(DummyMemory::new(size))
}
