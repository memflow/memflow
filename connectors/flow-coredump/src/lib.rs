mod native;
use native::*;

use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

use flow_core::*;

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

        let mem_map =
            try_parse_coredump64(&mut file).or_else(|_| try_parse_coredump32(&mut file))?;

        Ok(Self { file, mem_map })
    }
}

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
