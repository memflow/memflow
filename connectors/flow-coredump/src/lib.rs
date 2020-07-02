mod native;
use native::*;

use std::fs::OpenOptions;
use std::path::Path;

use flow_core::*;

#[cfg(not(feature = "memmap"))]
use std::fs::File;

#[cfg(not(feature = "memmap"))]
use std::io::{Read, Seek, SeekFrom};

#[cfg(not(feature = "memmap"))]
pub struct CoreDump {
    file: File,
    mem_map: MemoryMap,
}

#[cfg(not(feature = "memmap"))]
impl CoreDump {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let mut file = OpenOptions::new()
            .read(true)
            .write(false)
            .open(path)
            .map_err(|_| Error::Connector("unable to open coredump file"))?;

        let mem_map =
            try_parse_coredump64(&mut file).or_else(|_| try_parse_coredump32(&mut file))?;

        Ok(Self { file, mem_map })
    }
}

#[cfg(not(feature = "memmap"))]
impl PhysicalMemory for CoreDump {
    fn phys_read_raw_list(&mut self, data: &mut [PhysicalReadData]) -> Result<()> {
        for (addr, buf) in data.iter_mut() {
            let real_addr = self.mem_map.map(addr.address())?;
            self.file.seek(SeekFrom::Start(real_addr.as_u64())).ok();
            self.file.read_exact(buf).ok();
        }
        Ok(())
    }

    fn phys_write_raw_list(&mut self, _data: &[PhysicalWriteData]) -> Result<()> {
        Err(Error::Connector("write to coredump is not implemented"))
    }
}

#[cfg(feature = "memmap")]
use memmap::{Mmap, MmapOptions};

#[cfg(feature = "memmap")]
pub struct CoreDump {
    file_map: Mmap,
    mem_map: MemoryMap,
}

#[cfg(feature = "memmap")]
impl CoreDump {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let mut file = OpenOptions::new()
            .read(true)
            .write(false)
            .open(path)
            .map_err(|_| Error::Connector("unable to open coredump file"))?;

        let file_map = unsafe {
            MmapOptions::new()
                .map(&file)
                .map_err(|_| Error::Connector("unable to map coredump file"))?
        };

        let mem_map =
            try_parse_coredump64(&mut file).or_else(|_| try_parse_coredump32(&mut file))?;

        Ok(Self { file_map, mem_map })
    }
}

#[cfg(feature = "memmap")]
impl PhysicalMemory for CoreDump {
    fn phys_read_raw_list(&mut self, data: &mut [PhysicalReadData]) -> Result<()> {
        for (addr, buf) in data.iter_mut() {
            let real_addr = self.mem_map.map(addr.address())?;
            buf.copy_from_slice(
                &self.file_map[real_addr.as_usize()..(real_addr + buf.len()).as_usize()],
            );
        }
        Ok(())
    }

    fn phys_write_raw_list(&mut self, _data: &[PhysicalWriteData]) -> Result<()> {
        Err(Error::Connector("write to coredump is not implemented"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn parse_win10_64bit() {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("resources/test/coredump_win10_64bit_stripped.raw");
        CoreDump::open(path).unwrap();
    }

    #[test]
    fn parse_win7_32bit() {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("resources/test/coredump_win7_32bit_stripped.raw");
        CoreDump::open(path).unwrap();
    }
}
