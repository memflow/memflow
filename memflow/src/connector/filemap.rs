use crate::error::{Error, Result};
use crate::mem::MemoryMap;
use crate::types::Address;
use memmap::{Mmap, MmapMut, MmapOptions};

use std::fs::File;
use std::sync::Arc;

use super::mmap::MappedPhysicalMemory;

#[derive(Clone)]
pub struct MMAPInfo<'a> {
    mem_map: MemoryMap<&'a [u8]>,
    _buf: Arc<Mmap>,
}

impl<'a> AsRef<MemoryMap<&'a [u8]>> for MMAPInfo<'a> {
    fn as_ref(&self) -> &MemoryMap<&'a [u8]> {
        &self.mem_map
    }
}

impl<'a> MMAPInfo<'a> {
    pub fn try_with_filemap(file: File, map: MemoryMap<(Address, usize)>) -> Result<Self> {
        let file_map = unsafe {
            MmapOptions::new()
                .map(&file)
                .map_err(|_| Error::Connector("unable to map file"))?
        };

        Self::try_with_bufmap(file_map, map)
    }

    pub fn try_with_bufmap(buf: Mmap, map: MemoryMap<(Address, usize)>) -> Result<Self> {
        let mut new_map = MemoryMap::new();

        let buf_len = buf.as_ref().len();
        let buf_ptr = buf.as_ref().as_ptr();

        for (base, (output_base, size)) in map.into_iter() {
            if output_base.as_usize() >= buf_len {
                return Err(Error::Connector("Memory map is out of range"));
            }

            let output_end = std::cmp::min(output_base.as_usize() + size, buf_len);

            new_map.push(base, unsafe {
                std::slice::from_raw_parts(
                    buf_ptr.add(output_base.as_usize()),
                    output_end - output_base.as_usize(),
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

pub type ReadMappedFilePhysicalMemory<'a> = MappedPhysicalMemory<&'a [u8], MMAPInfo<'a>>;

pub struct MMAPInfoMut<'a> {
    mem_map: MemoryMap<&'a mut [u8]>,
    _buf: MmapMut,
}

impl<'a> AsRef<MemoryMap<&'a mut [u8]>> for MMAPInfoMut<'a> {
    fn as_ref(&self) -> &MemoryMap<&'a mut [u8]> {
        &self.mem_map
    }
}

impl<'a> MMAPInfoMut<'a> {
    pub fn try_with_filemap_mut(file: File, map: MemoryMap<(Address, usize)>) -> Result<Self> {
        let file_map = unsafe {
            MmapOptions::new()
                .map_mut(&file)
                .map_err(|_| Error::Connector("unable to map file"))?
        };

        Self::try_with_bufmap_mut(file_map, map)
    }

    pub fn try_with_bufmap_mut(mut buf: MmapMut, map: MemoryMap<(Address, usize)>) -> Result<Self> {
        let mut new_map = MemoryMap::new();

        let buf_len = buf.as_ref().len();
        let buf_ptr = buf.as_mut().as_mut_ptr();

        for (base, (output_base, size)) in map.into_iter() {
            if output_base.as_usize() >= buf_len {
                return Err(Error::Connector("Memory map is out of range"));
            }

            let output_end = std::cmp::min(output_base.as_usize() + size, buf_len);

            new_map.push(base, unsafe {
                std::slice::from_raw_parts_mut(
                    buf_ptr.add(output_base.as_usize()),
                    output_end - output_base.as_usize(),
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

pub type WriteMappedFilePhysicalMemory<'a> = MappedPhysicalMemory<&'a mut [u8], MMAPInfoMut<'a>>;
