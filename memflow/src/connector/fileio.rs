/*!
Basic connector which works on file i/o operations (`Seek`, `Read`, `Write`).
*/

use crate::error::{Error, ErrorKind, ErrorOrigin, Result};
use crate::mem::{
    phys_mem::*, MemData, MemoryMap, PhysicalMemory, PhysicalMemoryMetadata, PhysicalReadData,
    PhysicalWriteData,
};
use crate::types::{umem, Address};

use std::fs::File;
use std::io::{self, Read, Seek, SeekFrom, Write};
use std::ops::{Deref, DerefMut};

#[cfg(feature = "plugins")]
use crate::cglue::*;

/// File that implements Clone
///
/// This file is meant for use with FileIoMemory when clone is needed, and possible Clone panics
/// are acceptable (they should either always, or never happen on a given platform, probably never)
pub struct CloneFile {
    file: File,
}

impl Clone for CloneFile {
    /// Clone the file
    ///
    /// # Panics
    ///
    /// If file cloning fails.
    fn clone(&self) -> Self {
        Self {
            file: self.file.try_clone().unwrap(),
        }
    }
}

impl Deref for CloneFile {
    type Target = File;

    fn deref(&self) -> &Self::Target {
        &self.file
    }
}

impl DerefMut for CloneFile {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.file
    }
}

impl From<File> for CloneFile {
    fn from(file: File) -> Self {
        Self { file }
    }
}

impl Read for CloneFile {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.file.read(buf)
    }
}

impl Read for &CloneFile {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        (&self.file).read(buf)
    }
}

impl Seek for CloneFile {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        self.file.seek(pos)
    }
}

impl Seek for &CloneFile {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        (&self.file).seek(pos)
    }
}

impl Write for CloneFile {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.file.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.file.flush()
    }
}

impl Write for &CloneFile {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        (&self.file).write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        (&self.file).flush()
    }
}

/// Accesses physical memory via file i/o.
///
/// This backend helper works in tandem with MappedPhysicalMemory.
///
/// # Examples
/// ```
/// use memflow::connector::FileIoMemory;
/// use memflow::mem::MemoryMap;
///
/// use std::fs::File;
///
/// fn open(file: &File) {
///     let map = MemoryMap::new();
///     let connector = FileIoMemory::try_with_reader(file, map);
/// }
/// ```
#[derive(Clone)]
pub struct FileIoMemory<T> {
    reader: T,
    mem_map: MemoryMap<(Address, umem)>,
}

impl<T: Seek + Read + Write + Send> FileIoMemory<T> {
    pub fn try_with_reader(reader: T, mem_map: MemoryMap<(Address, umem)>) -> Result<Self> {
        Ok(Self { reader, mem_map })
    }
}

impl<T: Seek + Read + Write + Send> PhysicalMemory for FileIoMemory<T> {
    fn phys_read_raw_iter<'a>(
        &mut self,
        data: CIterator<PhysicalReadData<'a>>,
        out_fail: &mut PhysicalReadFailCallback<'_, 'a>,
    ) -> Result<()> {
        let mut void = |(addr, buf): (Address, _)| {
            // TODO: manage not to lose physical page information here???
            out_fail.call(MemData(addr.into(), buf))
        };
        for ((file_off, _), buf) in self.mem_map.map_iter(data.map(<_>::into), &mut void) {
            self.reader
                .seek(SeekFrom::Start(file_off.to_umem() as u64))
                .map_err(|err| {
                    Error(ErrorOrigin::Connector, ErrorKind::UnableToSeekFile).log_error(err)
                })?;
            self.reader.read_exact(buf.into()).map_err(|err| {
                Error(ErrorOrigin::Connector, ErrorKind::UnableToWriteFile).log_error(err)
            })?;
        }
        Ok(())
    }

    fn phys_write_raw_iter<'a>(
        &mut self,
        data: CIterator<PhysicalWriteData<'a>>,
        out_fail: &mut PhysicalWriteFailCallback<'_, 'a>,
    ) -> Result<()> {
        let void = &mut |(addr, buf): (Address, _)| {
            // TODO: manage not to lose physical page information here???
            out_fail.call(MemData(addr.into(), buf))
        };
        for ((file_off, _), buf) in self.mem_map.map_iter(data.map(<_>::from), void) {
            self.reader
                .seek(SeekFrom::Start(file_off.to_umem() as u64))
                .map_err(|err| {
                    Error(ErrorOrigin::Connector, ErrorKind::UnableToSeekFile).log_error(err)
                })?;
            self.reader.write(buf.into()).map_err(|err| {
                Error(ErrorOrigin::Connector, ErrorKind::UnableToWriteFile).log_error(err)
            })?;
        }
        Ok(())
    }

    fn metadata(&self) -> PhysicalMemoryMetadata {
        PhysicalMemoryMetadata {
            max_address: self.mem_map.max_address(),
            real_size: self.mem_map.real_size(),
            readonly: false,
            ideal_batch_size: u32::MAX,
        }
    }
}

cglue_impl_group!(
    FileIoMemory<T: Read + Seek + Write + Send>,
    crate::plugins::ConnectorInstance,
    {}
);
