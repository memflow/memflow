/*!
Basic connector which works on file i/o operations (`Seek`, `Read`, `Write`).
*/

use crate::error::{Error, ErrorKind, ErrorOrigin, Result};
use crate::mem::{
    opt_call, MemoryMap, PhysicalMemory, PhysicalMemoryMetadata, PhysicalReadMemOps,
    PhysicalWriteMemOps,
};
use crate::types::{umem, Address};

use std::fs::File;
use std::io::{self, Read, Seek, SeekFrom, Write};
use std::ops::{Deref, DerefMut};

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

#[allow(clippy::needless_option_as_deref)]
#[allow(clippy::collapsible_if)]
#[allow(clippy::blocks_in_if_conditions)]
impl<T: Seek + Read + Write + Send> PhysicalMemory for FileIoMemory<T> {
    fn phys_read_raw_iter(&mut self, mut data: PhysicalReadMemOps) -> Result<()> {
        let mut iter = self.mem_map.map_iter(data.inp, data.out_fail);
        while let Some(CTup3((file_off, _), meta_addr, mut buf)) = iter.next() {
            if self
                .reader
                .seek(SeekFrom::Start(file_off.to_umem() as u64))
                .map_err(|err| {
                    Error(ErrorOrigin::Connector, ErrorKind::UnableToSeekFile).log_error(err)
                })
                .is_ok()
            {
                if self
                    .reader
                    .read_exact(&mut *buf)
                    .map_err(|err| {
                        Error(ErrorOrigin::Connector, ErrorKind::UnableToReadFile).log_error(err)
                    })
                    .is_ok()
                {
                    opt_call(data.out.as_deref_mut(), CTup2(meta_addr, buf));
                    continue;
                }
            }
            opt_call(iter.fail_out(), CTup2(meta_addr, buf));
        }
        Ok(())
    }

    fn phys_write_raw_iter(&mut self, mut data: PhysicalWriteMemOps) -> Result<()> {
        let mut iter = self.mem_map.map_iter(data.inp, data.out_fail);
        while let Some(CTup3((file_off, _), meta_addr, buf)) = iter.next() {
            if self
                .reader
                .seek(SeekFrom::Start(file_off.to_umem() as u64))
                .map_err(|err| {
                    Error(ErrorOrigin::Connector, ErrorKind::UnableToSeekFile).log_error(err)
                })
                .is_ok()
            {
                if self
                    .reader
                    .write_all(&*buf)
                    .map_err(|err| {
                        Error(ErrorOrigin::Connector, ErrorKind::UnableToWriteFile).log_error(err)
                    })
                    .is_ok()
                {
                    opt_call(data.out.as_deref_mut(), CTup2(meta_addr, buf));
                    continue;
                }
            }
            opt_call(iter.fail_out(), CTup2(meta_addr, buf));
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
