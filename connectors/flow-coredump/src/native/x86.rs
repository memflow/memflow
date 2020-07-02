use super::*;

use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::mem::MaybeUninit;

use dataview::Pod;
use flow_core::*;
use log::{debug, info};

pub const DUMP_VALID_DUMP32: u32 = 0x504d5544;
pub const IMAGE_FILE_MACHINE_I386: u32 = 0x014c;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct PhysicalMemoryRun32 {
    pub base_page: u32,
    pub page_count: u32,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct PhysicalMemoryDescriptor32 {
    pub number_of_runs: u32,
    pub number_of_pages: u32,
    pub runs: [PhysicalMemoryRun32; PHYSICAL_MEMORY_MAX_RUNS],
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct CoreDumpHeader32 {
    pub signature: u32,                                    // 0x0000
    pub valid_dump: u32,                                   // 0x0004
    pub major_version: u32,                                // 0x0008
    pub minor_version: u32,                                // 0x000c
    pub directory_table_base: u32,                         // 0x0010
    pub pfn_data_base: u32,                                // 0x0014
    pub ps_loaded_module_list: u32,                        // 0x0018
    pub ps_active_process_head: u32,                       // 0x001c
    pub machine_image_type: u32,                           // 0x0020
    pub number_processors: u32,                            // 0x0024
    pub bug_check_code: u32,                               // 0x0028
    pub bug_check_parameter1: u32,                         // 0x002c
    pub bug_check_parameter2: u32,                         // 0x0030
    pub bug_check_parameter3: u32,                         // 0x0034
    pub bug_check_parameter4: u32,                         // 0x0038
    pub version_user: [u8; 32],                            // 0x003c
    pub pae_enabled: u8,                                   // 0x005c
    pub kd_secondary_version: u8,                          // 0x005d
    pub spare: [u8; 2],                                    // 0x005e
    pub kd_debugger_data_block: u32,                       // 0x0060
    pub physical_memory_block: PhysicalMemoryDescriptor32, // 0x0064
    pub pad0: [u8; 436],                                   //
    pub context_record: [u8; 1200],                        // 0x0320
    pub exception_record: [u8; 80],                        // EXCEPTION_RECORD32 - 0x07d0
    pub comment: [u8; 128],                                // 0x0820
    pub reserved0: [u8; 1768],                             // 0x08a0
    pub dump_type: u32,                                    // 0x0f88
    pub mini_dump_fields: u32,                             // 0x0f8c
    pub secondary_data_state: u32,                         // 0x0f90
    pub product_type: u32,                                 // 0x0f94
    pub suite_mask: u32,                                   // 0x0f98
    pub reserved1: [u8; 4],                                // 0x0f9c
    pub required_dump_space: u64,                          // 0x0fa0
    pub reserved2: [u8; 16],                               // 0x0fa8
    pub system_up_time: u64,                               // 0x0fb8
    pub system_time: u64,                                  // 0x0fc0
    pub reserved3: [u8; 56],                               // 0x0fc8
} // size: 0x1000

#[allow(clippy::uninit_assumed_init)]
impl CoreDumpHeader32 {
    pub fn uninit() -> Self {
        unsafe { MaybeUninit::uninit().assume_init() }
    }
}

unsafe impl Pod for CoreDumpHeader32 {}

pub fn try_parse_coredump32(file: &mut File) -> Result<MemoryMap> {
    let mut header = CoreDumpHeader32::uninit();

    file.seek(SeekFrom::Start(0))
        .map_err(|_| Error::Connector("unable to seek to coredump 64 header"))?;

    file.read_exact(header.as_bytes_mut())
        .map_err(|_| Error::Connector("unable to read coredump 32 header"))?;

    if header.signature != DUMP_SIGNATURE {
        return Err(Error::Connector("header signature is not valid"));
    }

    if header.valid_dump != DUMP_VALID_DUMP32 {
        return Err(Error::Connector("header dump flag is not valid"));
    }

    if header.dump_type != DUMP_TYPE_FULL {
        return Err(Error::Connector(
            "invalid dump type, only full dumps are supported",
        ));
    }

    // TODO: dynamic switching with support for both archs
    if header.machine_image_type != IMAGE_FILE_MACHINE_I386 {
        return Err(Error::Connector("invalid machine image type"));
    }

    info!("32-bit Microsoft Crash Dump verified");

    if header.physical_memory_block.number_of_runs > PHYSICAL_MEMORY_MAX_RUNS as u32 {
        return Err(Error::Connector(
            "too many memory segments in crash dump file",
        ));
    }

    let mut mem_map = MemoryMap::new();

    let mut real_base = 0x1000;

    // TODO: dirty fix until we fixed ranges in mem_map
    mem_map.push_range(Address::NULL, 0x1000.into(), Address::NULL);

    for i in 0..header.physical_memory_block.number_of_runs {
        let base = header.physical_memory_block.runs[i as usize].base_page << 12;
        let size = header.physical_memory_block.runs[i as usize].page_count << 12;

        debug!(
            "adding memory mapping: base={:x} size={:x} real_base={:x}",
            base, size, real_base
        );
        mem_map.push(base.into(), size as usize, real_base.into());

        real_base += size;
    }

    // TODO: if the file contains no runs set a range from 0 to 0xffffffff
    Ok(mem_map)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem::size_of;

    #[test]
    fn test_struct_sizes_x86() {
        assert_eq!(size_of::<PhysicalMemoryRun32>(), 0x8);
        assert_eq!(size_of::<PhysicalMemoryDescriptor32>(), 0x108);
        assert_eq!(size_of::<CoreDumpHeader32>(), 0x1000);
    }

    #[test]
    fn test_struct_members_x86() {
        let header = CoreDumpHeader32::uninit();
        assert_eq!(
            &header.version_user as *const _ as usize - &header as *const _ as usize,
            0x3c
        );
        assert_eq!(
            &header.kd_debugger_data_block as *const _ as usize - &header as *const _ as usize,
            0x60
        );
        assert_eq!(
            &header.physical_memory_block as *const _ as usize - &header as *const _ as usize,
            0x64
        );
        assert_eq!(
            &header.context_record as *const _ as usize - &header as *const _ as usize,
            0x320
        );
        assert_eq!(
            &header.exception_record as *const _ as usize - &header as *const _ as usize,
            0x7d0
        );
        assert_eq!(
            &header.dump_type as *const _ as usize - &header as *const _ as usize,
            0xf88
        );
    }
}
