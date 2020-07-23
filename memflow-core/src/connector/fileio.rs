use crate::error::{Error, Result};
use crate::iter::FnExtend;
use crate::mem::{MemoryMap, PhysicalMemory, PhysicalReadData, PhysicalWriteData};
use crate::types::Address;

use std::io::{Read, Seek, SeekFrom, Write};

pub struct IOPhysicalMemory<T> {
    reader: T,
    mem_map: MemoryMap<(Address, usize)>,
}

impl<T: Seek + Read + Write> IOPhysicalMemory<T> {
    pub fn try_with_reader(reader: T, mem_map: MemoryMap<(Address, usize)>) -> Result<Self> {
        Ok(Self { reader, mem_map })
    }
}

impl<T: Seek + Read + Write> PhysicalMemory for IOPhysicalMemory<T> {
    fn phys_read_raw_list(&mut self, data: &mut [PhysicalReadData]) -> Result<()> {
        let mut void = FnExtend::void();
        for ((file_off, _), buf) in self.mem_map.map_iter(
            data.iter_mut().map(|(addr, buf)| (*addr, &mut **buf)),
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
        for ((file_off, _), buf) in self.mem_map.map_iter(data.iter().copied(), &mut void) {
            self.reader
                .seek(SeekFrom::Start(file_off.as_u64()))
                .map_err(|_| Error::Connector("Seek failed"))?;
            self.reader
                .write(buf)
                .map_err(|_| Error::Connector("Write failed"))?;
        }
        Ok(())
    }
}
