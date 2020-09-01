/*!
Basic connector which works on file i/o operations (`Seek`, `Read`, `Write`).
*/

use crate::error::{Error, Result};
use crate::iter::FnExtend;
use crate::mem::{
    MemoryMap, PhysicalMemory, PhysicalMemoryMetadata, PhysicalReadData, PhysicalWriteData,
};
use crate::types::Address;

use std::io::{Read, Seek, SeekFrom, Write};

/// Accesses physical memory via file i/o.
///
/// This backend helper works in tandem with MappedPhysicalMemory.
///
/// # Examples
/// ```
/// use memflow::connector::FileIOMemory;
/// use memflow::mem::MemoryMap;
///
/// use std::fs::File;
///
/// fn open(file: &File) {
///     let map = MemoryMap::new();
///     let connector = FileIOMemory::try_with_reader(file, map);
/// }
/// ```
#[derive(Clone)]
pub struct FileIOMemory<T> {
    reader: T,
    mem_map: MemoryMap<(Address, usize)>,
}

impl<T: Seek + Read + Write + Send> FileIOMemory<T> {
    pub fn try_with_reader(reader: T, mem_map: MemoryMap<(Address, usize)>) -> Result<Self> {
        Ok(Self { reader, mem_map })
    }
}

impl<T: Seek + Read + Write + Send> PhysicalMemory for FileIOMemory<T> {
    fn phys_read_raw_list(&mut self, data: &mut [PhysicalReadData]) -> Result<()> {
        let mut void = FnExtend::void();
        for ((file_off, _), buf) in self.mem_map.map_iter(
            data.iter_mut()
                .map(|PhysicalReadData(addr, buf)| (*addr, &mut **buf)),
            &mut void,
        ) {
            self.reader
                .seek(SeekFrom::Start(file_off.as_u64()))
                .map_err(|_| Error::Connector("Seek failed"))?;
            self.reader
                .read_exact(buf)
                .map_err(|_| Error::Connector("Read failed"))?;
        }
        Ok(())
    }

    fn phys_write_raw_list(&mut self, data: &[PhysicalWriteData]) -> Result<()> {
        let mut void = FnExtend::void();
        for ((file_off, _), buf) in self
            .mem_map
            .map_iter(data.iter().copied().map(<_>::from), &mut void)
        {
            self.reader
                .seek(SeekFrom::Start(file_off.as_u64()))
                .map_err(|_| Error::Connector("Seek failed"))?;
            self.reader
                .write(buf)
                .map_err(|_| Error::Connector("Write failed"))?;
        }
        Ok(())
    }

    fn metadata(&self) -> PhysicalMemoryMetadata {
        PhysicalMemoryMetadata {
            size: self
                .mem_map
                .as_ref()
                .iter()
                .last()
                .map(|map| map.base().as_usize() + map.output().1)
                .unwrap(),
            readonly: false,
        }
    }
}
