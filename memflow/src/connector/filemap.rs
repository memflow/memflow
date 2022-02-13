use crate::error::{Error, ErrorKind, ErrorOrigin, Result};
use crate::mem::MemoryMap;
use crate::types::{umem, Address};
use memmap::{Mmap, MmapMut, MmapOptions};

use core::convert::TryInto;
use std::fs::File;
use std::sync::Arc;

use super::mmap::MappedPhysicalMemory;

#[derive(Clone)]
pub struct MmapInfo<'a> {
    mem_map: MemoryMap<&'a [u8]>,
    _buf: Arc<Mmap>,
}

impl<'a> AsRef<MemoryMap<&'a [u8]>> for MmapInfo<'a> {
    fn as_ref(&self) -> &MemoryMap<&'a [u8]> {
        &self.mem_map
    }
}

impl<'a> MmapInfo<'a> {
    pub fn try_with_filemap(file: File, map: MemoryMap<(Address, umem)>) -> Result<Self> {
        let file_map = unsafe {
            MmapOptions::new().map(&file).map_err(|err| {
                Error(ErrorOrigin::Connector, ErrorKind::UnableToMapFile).log_error(err)
            })?
        };

        Self::try_with_bufmap(file_map, map)
    }

    pub fn try_with_bufmap(buf: Mmap, map: MemoryMap<(Address, umem)>) -> Result<Self> {
        let mut new_map = MemoryMap::new();

        let buf_len = buf.as_ref().len() as umem;
        let buf_ptr = buf.as_ref().as_ptr();

        for (base, (output_base, size)) in map.into_iter() {
            let output_base_umem = output_base.to_umem();
            if output_base_umem >= buf_len {
                return Err(Error(
                    ErrorOrigin::Connector,
                    ErrorKind::MemoryMapOutOfRange,
                ));
            }

            let output_end = std::cmp::min(output_base_umem + size, buf_len);

            new_map.push(base, unsafe {
                std::slice::from_raw_parts(
                    buf_ptr.add(output_base_umem.try_into().unwrap()),
                    (output_end - output_base_umem).try_into().unwrap(),
                )
            });
        }

        Ok(Self {
            mem_map: new_map,
            _buf: Arc::new(buf),
        })
    }

    pub fn into_connector(self) -> ReadMappedFilePhysicalMemory<'a> {
        MappedPhysicalMemory::with_info(self)
    }
}

pub type ReadMappedFilePhysicalMemory<'a> = MappedPhysicalMemory<&'a [u8], MmapInfo<'a>>;

pub struct MmapInfoMut<'a> {
    mem_map: MemoryMap<&'a mut [u8]>,
    _buf: MmapMut,
}

impl<'a> AsRef<MemoryMap<&'a mut [u8]>> for MmapInfoMut<'a> {
    fn as_ref(&self) -> &MemoryMap<&'a mut [u8]> {
        &self.mem_map
    }
}

impl<'a> MmapInfoMut<'a> {
    pub fn try_with_filemap_mut(file: File, map: MemoryMap<(Address, umem)>) -> Result<Self> {
        let file_map = unsafe {
            MmapOptions::new().map_mut(&file).map_err(|err| {
                Error(ErrorOrigin::Connector, ErrorKind::UnableToMapFile).log_error(err)
            })?
        };

        Self::try_with_bufmap_mut(file_map, map)
    }

    pub fn try_with_bufmap_mut(mut buf: MmapMut, map: MemoryMap<(Address, umem)>) -> Result<Self> {
        let mut new_map = MemoryMap::new();

        let buf_len = buf.as_ref().len() as umem;
        let buf_ptr = buf.as_mut().as_mut_ptr();

        for (base, (output_base, size)) in map.into_iter() {
            let output_base_umem = output_base.to_umem();
            if output_base_umem >= buf_len as umem {
                return Err(Error(
                    ErrorOrigin::Connector,
                    ErrorKind::MemoryMapOutOfRange,
                ));
            }

            let output_end = std::cmp::min(output_base_umem + size, buf_len);

            new_map.push(base, unsafe {
                std::slice::from_raw_parts_mut(
                    buf_ptr.add(output_base_umem.try_into().unwrap()),
                    (output_end - output_base_umem).try_into().unwrap(),
                )
            });
        }

        Ok(Self {
            mem_map: new_map,
            _buf: buf,
        })
    }

    pub fn into_connector(self) -> WriteMappedFilePhysicalMemory<'a> {
        MappedPhysicalMemory::with_info(self)
    }
}

pub type WriteMappedFilePhysicalMemory<'a> = MappedPhysicalMemory<&'a mut [u8], MmapInfoMut<'a>>;
