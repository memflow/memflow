mod native;
use native::*;

use std::fs::OpenOptions;
use std::path::Path;

use flow_core::*;

#[cfg(not(feature = "memmap"))]
use std::fs::File;

#[cfg(not(feature = "memmap"))]
use std::io::{Read, Seek, SeekFrom};

#[cfg(feature = "memmap")]
use memmap::{Mmap, MmapOptions};

/**
The `CoreDump` struct implements a physical memory backend that reads Microsoft Windows Coredump files.

When opening a crashdump it tries to parse the first 0x2000 bytes of the file as a 64 bit Windows Coredump.
If the validation of the 64 bit Header fails it tries to read the first 0x1000 bytes of the file and validates it as 32 bit Windows Coredump.

If neither attempt succeeds the function will fail with an `Error::Conector` error.

# Examples

```
use std::path::PathBuf;
use flow_coredump::CoreDump;

let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    .join("resources/test/coredump_win10_64bit_stripped.raw");
let mut mem = CoreDump::open(path).unwrap();
```
*/
pub struct CoreDump {
    #[cfg(not(feature = "memmap"))]
    file: File,
    #[cfg(feature = "memmap")]
    file_map: Mmap,
    mem_map: MemoryMap<(Address, usize)>,
}

impl CoreDump {
    /// Opens a Microsoft Windows Coredump.
    /// This function will use regular file operations for accessing the file's contents.
    /// For a higher performance version see the 'memmap' feature of this crate.
    #[cfg(not(feature = "memmap"))]
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let mut file = OpenOptions::new()
            .read(true)
            .write(false)
            .open(path)
            .map_err(|_| Error::Connector("unable to open coredump file"))?;

        let mem_map = parse_coredump64(&mut file).or_else(|_| parse_coredump32(&mut file))?;

        Ok(Self { file, mem_map })
    }

    /// Opens a Microsoft Windows Coredump.
    /// This function will use a file-backed memory map for accessing the file's contents.
    #[cfg(feature = "memmap")]
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

        let mem_map = parse_coredump64(&mut file).or_else(|_| parse_coredump32(&mut file))?;

        Ok(Self { file_map, mem_map })
    }
}

impl PhysicalMemory for CoreDump {
    #[cfg(not(feature = "memmap"))]
    fn phys_read_raw_list(&mut self, data: &mut [PhysicalReadData]) -> Result<()> {
        let mut void = ExtendVoid::void();
        for (real_addr, buf) in self.mem_map.map_iter(
            data.iter_mut().map(|(addr, buf)| (*addr, &mut **buf)),
            &mut void,
        ) {
            self.file.seek(SeekFrom::Start(real_addr.as_u64())).ok();
            self.file.read_exact(buf).ok();
        }
        Ok(())
    }

    #[cfg(feature = "memmap")]
    fn phys_read_raw_list(&mut self, data: &mut [PhysicalReadData]) -> Result<()> {
        let mut void = ExtendVoid::void();
        for ((real_addr, _), buf) in self.mem_map.map_iter(
            data.iter_mut().map(|(addr, buf)| (*addr, &mut **buf)),
            &mut void,
        ) {
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
