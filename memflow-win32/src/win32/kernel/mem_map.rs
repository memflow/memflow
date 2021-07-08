use std::prelude::v1::*;

use log::{info, trace};
use std::fmt;

use memflow::mem::{MemoryMap, MemoryView};
use memflow::types::{size, Address};

use memflow::dataview::Pod;

const SIZE_4KB: u64 = size::kb(4) as u64;

/// The number of PhysicalMemoryRuns contained in the Header
pub const PHYSICAL_MEMORY_MAX_RUNS: usize = 32;

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct PhysicalMemoryRun<T: Pod + fmt::Debug> {
    pub base_page: T,
    pub page_count: T,
}
unsafe impl<T: Pod + fmt::Debug> Pod for PhysicalMemoryRun<T> {}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct PhysicalMemoryDescriptor<T: Pod + fmt::Debug> {
    pub number_of_runs: u32,
    pub number_of_pages: T,
    pub runs: [PhysicalMemoryRun<T>; PHYSICAL_MEMORY_MAX_RUNS],
}
unsafe impl<T: Pod + fmt::Debug> Pod for PhysicalMemoryDescriptor<T> {}
const _: [(); std::mem::size_of::<PhysicalMemoryDescriptor<u32>>()] = [(); 0x108];
const _: [(); std::mem::size_of::<PhysicalMemoryDescriptor<u64>>()] = [(); 0x210];

pub fn parse<T: MemoryView, U: Pod + Copy + fmt::Debug + fmt::LowerHex + Into<u64>>(
    virt_mem: &mut T,
    descriptor_ptr_ptr: Address,
) -> Option<MemoryMap<(Address, usize)>> {
    let descriptor_ptr = virt_mem.read_addr64(descriptor_ptr_ptr).ok()?;

    trace!("found phys_mem_block pointer at: {}", descriptor_ptr);
    let descriptor: PhysicalMemoryDescriptor<U> = virt_mem.read(descriptor_ptr).ok()?;

    trace!("found phys_mem_block: {:?}", descriptor);
    if descriptor.number_of_runs <= PHYSICAL_MEMORY_MAX_RUNS as u32 {
        let mut mem_map = MemoryMap::new();

        for i in 0..descriptor.number_of_runs {
            let base = descriptor.runs[i as usize].base_page.into() * SIZE_4KB;
            let size = descriptor.runs[i as usize].page_count.into() * SIZE_4KB;

            trace!("adding memory mapping: base={:x} size={:x}", base, size);
            mem_map.push_remap(base.into(), size as usize, Address::from(base));
        }

        Some(mem_map)
    } else {
        info!(
            "too many memory segments in phys_mem_block: {} found, {} expected",
            descriptor.number_of_runs, PHYSICAL_MEMORY_MAX_RUNS
        );
        None
    }
}
