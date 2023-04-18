use std::alloc::{alloc, Layout};

use crate::cglue::*;
use crate::connector::MappedPhysicalMemory;
use crate::derive::connector;
use crate::error::{Error, ErrorKind, ErrorOrigin, Result};
use crate::mem::mem_data::*;
use crate::mem::{MemoryMap, PhysicalMemory, PhysicalMemoryMapping, PhysicalMemoryMetadata};
use crate::plugins::*;
use crate::types::{size, umem, Address};

cglue_impl_group!(DummyMemory, ConnectorInstance, {});

pub struct DummyMemory {
    pub(crate) buf: Box<[u8]>,
    pub(crate) mem: MappedPhysicalMemory<&'static mut [u8], MemoryMap<&'static mut [u8]>>,
}

impl DummyMemory {
    pub fn new(size: usize) -> Self {
        let mem_layout = Layout::from_size_align(size, 0x1000).unwrap();
        let mem_ptr = unsafe { alloc(mem_layout) };
        let buf = unsafe { Vec::from_raw_parts(mem_ptr, size, size) }.into_boxed_slice();

        let mut map = MemoryMap::new();
        map.push_range(
            Address::null(),
            buf.len().into(),
            (buf.as_ptr() as umem).into(),
        );

        let buf_mem = unsafe { MappedPhysicalMemory::from_addrmap_mut(map) };

        Self { buf, mem: buf_mem }
    }
}

impl Clone for DummyMemory {
    fn clone(&self) -> Self {
        let mut map = MemoryMap::new();
        map.push_range(
            Address::null(),
            self.buf.len().into(),
            (self.buf.as_ptr() as usize).into(),
        );

        let mem = unsafe { MappedPhysicalMemory::from_addrmap_mut(map) };

        // create a aligned copy of self.buf
        let mem_layout = Layout::from_size_align(self.buf.len(), 0x1000).unwrap();
        let mem_ptr = unsafe { alloc(mem_layout) };
        let mut buf = unsafe { Vec::from_raw_parts(mem_ptr, self.buf.len(), self.buf.len()) }
            .into_boxed_slice();
        buf.copy_from_slice(self.buf.as_ref());

        Self { buf, mem }
    }
}

impl PhysicalMemory for DummyMemory {
    #[inline]
    fn phys_read_raw_iter(&mut self, data: PhysicalReadMemOps) -> Result<()> {
        self.mem.phys_read_raw_iter(data)
    }

    #[inline]
    fn phys_write_raw_iter(&mut self, data: PhysicalWriteMemOps) -> Result<()> {
        self.mem.phys_write_raw_iter(data)
    }

    #[inline]
    fn metadata(&self) -> PhysicalMemoryMetadata {
        self.mem.metadata()
    }

    #[inline]
    fn set_mem_map(&mut self, mem_map: &[PhysicalMemoryMapping]) {
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

#[connector(name = "dummy")]
pub fn create_connector(args: &ConnectorArgs) -> Result<DummyMemory> {
    let size = parse_size(&args.extra_args)?;
    Ok(DummyMemory::new(size))
}
