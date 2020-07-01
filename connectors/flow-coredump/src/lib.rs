use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom};
use std::mem::MaybeUninit;
use std::path::Path;

use log::{debug, info};

use flow_core::*;

use dataview::Pod;

const DUMP_SIGNATURE: u32 = 0x45474150;
//const DUMP_VALID_DUMP: u32 = 0x504d5544;
const DUMP_VALID_DUMP64: u32 = 0x34365544;
const DUMP_TYPE_FULL: u32 = 1;
//const IMAGE_FILE_MACHINE_I386: u32 = 0x014c;
const IMAGE_FILE_MACHINE_AMD64: u32 = 0x8664;

const PHYSICAL_MEMORY_MAX_RUNS: usize = 0x20;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct PhysicalMemoryRun64 {
    pub base_page: u64,
    pub page_count: u64,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct PhysicalMemoryDescriptor64 {
    pub number_of_runs: u32,
    reserved0: u32,
    pub number_of_pages: u32,
    reserved1: u32,
    pub runs: [PhysicalMemoryRun64; PHYSICAL_MEMORY_MAX_RUNS],
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct CoreDumpHeader64 {
    signature: u32,                                    // 0x0000
    valid_dump: u32,                                   // 0x0004
    major_version: u32,                                // 0x0008
    minor_version: u32,                                // 0x000c
    directory_table_base: u64,                         // 0x0010
    pfn_data_base: u64,                                // 0x0018
    ps_loaded_module_list: u64,                        // 0x0020
    ps_active_process_head: u64,                       // 0x0028
    machine_image_type: u32,                           // 0x0030
    number_processors: u32,                            // 0x0034
    bug_check_code: u32,                               // 0x0038
    bug_check_parameter1: u64,                         // 0x0040
    bug_check_parameter2: u64,                         // 0x0048
    bug_check_parameter3: u64,                         // 0x0050
    bug_check_parameter4: u64,                         // 0x0058
    version_user: [u8; 32],                            // 0x0060
    kd_debugger_data_block: u64,                       // 0x0080
    physical_memory_block: PhysicalMemoryDescriptor64, // 0x0088
    pad0: [u8; 176],                                   // 0x0344
    context_record: [u8; 3000],                        // 0x0348
    exception_record: [u8; 152],                       // EXCEPTION_RECORD64 - 0x0F00
    dump_type: u32,                                    // 0x0F98 // rust: 0xf94
    required_dump_space: u64,                          // 0x0FA0
    system_time: u64,                                  // 0x0FA8
    comment: [i8; 0x80],                               // 0x0FB0 May not be present.
    system_up_time: u64,                               // 0x1030
    mini_dump_fields: u32,                             // 0x1038
    secondary_data_state: u32,                         // 0x103c
    product_type: u32,                                 // 0x1040
    suite_mask: u32,                                   // 0x1044
    writer_status: u32,                                // 0x1048
    unused0: u8,                                       // 0x104c
    kd_secondary_version: u8, // 0x104d Present only for W2K3 SP1 and better
    unused1: [u8; 2],         // 0x104e
    reserved0: [u8; 4016],    // 0x1050
} // size: 0x2000

#[allow(clippy::uninit_assumed_init)]
impl CoreDumpHeader64 {
    pub fn uninit() -> Self {
        unsafe { MaybeUninit::uninit().assume_init() }
    }
}

unsafe impl Pod for CoreDumpHeader64 {}

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem::size_of;

    #[test]
    fn test_struct_sizes() {
        assert_eq!(size_of::<PhysicalMemoryRun64>(), 0x10);
        assert_eq!(size_of::<PhysicalMemoryDescriptor64>(), 0x210);
        assert_eq!(size_of::<CoreDumpHeader64>(), 0x2000);
    }

    #[test]
    fn test_struct_members() {
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

pub struct CoreDump {
    file: File,
    mem_map: MemoryMap,
}

impl CoreDump {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        // TODO: use mem map / feature/optional ?
        let mut file = OpenOptions::new()
            .read(true)
            .write(false)
            .open(path)
            .map_err(|_| Error::Connector("unable to open coredump file"))?;

        let mut header = CoreDumpHeader64::uninit();
        file.read_exact(header.as_bytes_mut())
            .map_err(|_| Error::Connector("unable to read coredump 64 header"))?;

        if header.signature != DUMP_SIGNATURE {
            return Err(Error::Connector("header signature is not valid"));
        }

        if header.valid_dump != DUMP_VALID_DUMP64 {
            return Err(Error::Connector("header dump flag is not valid"));
        }

        if header.dump_type != DUMP_TYPE_FULL {
            return Err(Error::Connector(
                "invalid dump type, only full dumps are supported",
            ));
        }

        // TODO: dynamic switching with support for both archs
        if header.machine_image_type != IMAGE_FILE_MACHINE_AMD64 {
            return Err(Error::Connector("invalid machine image type"));
        }

        info!("64-bit Microsoft Crash Dump verified");

        if header.physical_memory_block.number_of_runs > PHYSICAL_MEMORY_MAX_RUNS as u32 {
            return Err(Error::Connector(
                "too many memory segments in crash dump file",
            ));
        }

        let mut mem_map = MemoryMap::new();

        let mut real_base = 0x2000;

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

        // TODO: if the file contains no runs set a range from 0 to 0x0000ffffffffffff

        Ok(Self { file, mem_map })
    }
}

impl PhysicalMemory for CoreDump {
    fn phys_read_iter<'a, PI: PhysicalReadIterator<'a>>(&'a mut self, iter: PI) -> Result<()> {
        for (addr, out) in iter {
            let real_addr = self.mem_map.map(addr.address())?;
            self.file.seek(SeekFrom::Start(real_addr.as_u64())).ok();
            self.file.read_exact(out).ok();
        }
        Ok(())
    }

    fn phys_write_iter<'a, PI: PhysicalWriteIterator<'a>>(
        &'a mut self,
        mut _iter: PI,
    ) -> Result<()> {
        Err(Error::Connector("write to coredump is not implemented"))
    }
}
