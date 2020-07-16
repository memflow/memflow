use super::full_memory_dump::*;
use super::*;

use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::mem::size_of;
use std::mem::MaybeUninit;

use dataview::Pod;
use log::info;
use memflow_core::*;

pub const DUMP_VALID_DUMP64: u32 = 0x34365544;
pub const IMAGE_FILE_MACHINE_AMD64: u32 = 0x8664;

/// A 64bit Microsoft Windows Coredump Header
#[repr(C)]
#[derive(Copy, Clone)]
pub struct CoreDumpHeader64 {
    pub signature: u32,                                       // 0x0000
    pub valid_dump: u32,                                      // 0x0004
    pub major_version: u32,                                   // 0x0008
    pub minor_version: u32,                                   // 0x000c
    pub directory_table_base: u64,                            // 0x0010
    pub pfn_data_base: u64,                                   // 0x0018
    pub ps_loaded_module_list: u64,                           // 0x0020
    pub ps_active_process_head: u64,                          // 0x0028
    pub machine_image_type: u32,                              // 0x0030
    pub number_processors: u32,                               // 0x0034
    pub bug_check_code: u32,                                  // 0x0038
    pub bug_check_parameter1: u64,                            // 0x0040
    pub bug_check_parameter2: u64,                            // 0x0048
    pub bug_check_parameter3: u64,                            // 0x0050
    pub bug_check_parameter4: u64,                            // 0x0058
    pub version_user: [u8; 32],                               // 0x0060
    pub kd_debugger_data_block: u64,                          // 0x0080
    pub physical_memory_block: PhysicalMemoryDescriptor<u64>, // 0x0088
    pub pad0: [u8; 176],                                      // 0x0344
    pub context_record: [u8; 3000],                           // 0x0348
    pub exception_record: [u8; 152],                          // EXCEPTION_RECORD64 - 0x0F00
    pub dump_type: u32,                                       // 0x0F98
    pub required_dump_space: u64,                             // 0x0FA0
    pub system_time: u64,                                     // 0x0FA8
    pub comment: [i8; 0x80],                                  // 0x0FB0 May not be present.
    pub system_up_time: u64,                                  // 0x1030
    pub mini_dump_fields: u32,                                // 0x1038
    pub secondary_data_state: u32,                            // 0x103c
    pub product_type: u32,                                    // 0x1040
    pub suite_mask: u32,                                      // 0x1044
    pub writer_status: u32,                                   // 0x1048
    pub unused0: u8,                                          // 0x104c
    pub kd_secondary_version: u8,                             // 0x104d only on W2K3 SP1 and up
    pub unused1: [u8; 2],                                     // 0x104e
    pub reserved0: [u8; 4016],                                // 0x1050
} // size: 0x2000

#[allow(clippy::uninit_assumed_init)]
impl CoreDumpHeader64 {
    pub fn uninit() -> Self {
        unsafe { MaybeUninit::uninit().assume_init() }
    }
}

unsafe impl Pod for CoreDumpHeader64 {}

/// Tries to parse a file handle as a Microsoft Windows 64bit Coredump.
pub fn parse_coredump64(file: &mut File) -> Result<MemoryMap<(Address, usize)>> {
    let mut header = CoreDumpHeader64::uninit();

    file.seek(SeekFrom::Start(0))
        .map_err(|_| Error::Connector("unable to seek to coredump 64 header"))?;

    file.read_exact(header.as_bytes_mut())
        .map_err(|_| Error::Connector("unable to read coredump 64 header"))?;

    if header.signature != DUMP_SIGNATURE {
        return Err(Error::Connector("header signature is not valid"));
    }

    if header.valid_dump != DUMP_VALID_DUMP64 {
        return Err(Error::Connector("header dump flag is not valid"));
    }

    if header.machine_image_type != IMAGE_FILE_MACHINE_AMD64 {
        return Err(Error::Connector("invalid machine image type"));
    }

    info!("64-bit Microsoft Crash Dump verified");

    match header.dump_type {
        dump_type::FULL => full_memory_dump::parse_full_dump(
            header.physical_memory_block,
            size_of::<CoreDumpHeader64>(),
        ),
        dump_type::BIT_MAP => bitmap_dump::parse_bitmap_dump(file),
        _ => Err(Error::Connector(
            "invalid dump type, only full and bitmap dumps are supported",
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem::size_of;

    #[test]
    fn test_struct_sizes_x64() {
        assert_eq!(size_of::<PhysicalMemoryRun<u64>>(), 0x10);
        assert_eq!(size_of::<PhysicalMemoryDescriptor<u64>>(), 0x210);
        assert_eq!(size_of::<CoreDumpHeader64>(), 0x2000);
    }

    #[test]
    fn test_struct_members_x64() {
        let header = CoreDumpHeader64::uninit();
        assert_eq!(
            &header.version_user as *const _ as usize - &header as *const _ as usize,
            0x60
        );
        assert_eq!(
            &header.kd_debugger_data_block as *const _ as usize - &header as *const _ as usize,
            0x80
        );
        assert_eq!(
            &header.physical_memory_block as *const _ as usize - &header as *const _ as usize,
            0x88
        );
        assert_eq!(
            &header.context_record as *const _ as usize - &header as *const _ as usize,
            0x348
        );
        assert_eq!(
            &header.exception_record as *const _ as usize - &header as *const _ as usize,
            0xf00
        );
        assert_eq!(
            &header.dump_type as *const _ as usize - &header as *const _ as usize,
            0xf98
        );
    }
}
