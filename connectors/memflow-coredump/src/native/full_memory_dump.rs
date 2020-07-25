use super::*;

use std::convert::Into;

use dataview::Pod;
use log::debug;
use memflow_core::*;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct PhysicalMemoryRun<T: Pod> {
    pub base_page: T,
    pub page_count: T,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct PhysicalMemoryDescriptor<T: Pod> {
    pub number_of_runs: u32,
    pub number_of_pages: T,
    pub runs: [PhysicalMemoryRun<T>; PHYSICAL_MEMORY_MAX_RUNS],
}

pub fn parse_full_dump<T: Copy + Pod + Into<u64>>(
    descriptor: PhysicalMemoryDescriptor<T>,
    header_size: usize,
) -> Result<MemoryMap<(Address, usize)>> {
    let number_of_runs = descriptor.number_of_runs.into();

    if number_of_runs > PHYSICAL_MEMORY_MAX_RUNS as u64 {
        return Err(Error::Connector(
            "too many memory segments in crash dump file",
        ));
    }

    let mut mem_map = MemoryMap::new();

    // start runs from right after header size (x86: 0x1000 / x64: 0x2000)
    let mut real_base = header_size as u64;

    for i in 0..number_of_runs {
        let base = descriptor.runs[i as usize].base_page.into() << 12;
        let size = descriptor.runs[i as usize].page_count.into() << 12;

        debug!(
            "adding memory mapping: base={:x} size={:x} real_base={:x}",
            base, size, real_base
        );
        mem_map.push_remap(base.into(), size as usize, real_base.into());

        real_base += size;
    }

    // TODO: if the file contains no runs set a range from 0 to 0x0000ffffffffffff
    Ok(mem_map)
}
