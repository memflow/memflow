/*!
Basic connector which works on mapped memory.
*/

use crate::error::{Error, Result};
use crate::iter::FnExtend;
use crate::mem::{
    MemoryMap, PhysicalMemory, PhysicalMemoryMetadata, PhysicalReadData, PhysicalWriteData,
};
use crate::types::Address;

pub struct MappedPhysicalMemory<T, F> {
    info: F,
    marker: std::marker::PhantomData<T>,
}

impl<T, F: Clone> Clone for MappedPhysicalMemory<T, F> {
    fn clone(&self) -> Self {
        Self {
            info: self.info.clone(),
            marker: Default::default(),
        }
    }
}

impl MappedPhysicalMemory<&'static mut [u8], MemoryMap<&'static mut [u8]>> {
    /// Create a connector using virtual address mappings
    ///
    /// # Safety
    ///
    /// This connector assumes the memory map is valid, and writeable. Failure for these conditions
    /// to be met leads to undefined behaviour (most likely a segfault) when reading/writing.
    pub unsafe fn from_addrmap_mut(map: MemoryMap<(Address, usize)>) -> Self {
        let mut ret_map = MemoryMap::new();

        map.into_iter()
            .map(|(base, (real_base, size))| {
                (
                    base,
                    std::slice::from_raw_parts_mut(real_base.as_u64() as _, size),
                )
            })
            .for_each(|(base, buf)| {
                ret_map.push(base, buf);
            });

        Self::with_info(ret_map)
    }
}

impl MappedPhysicalMemory<&'static [u8], MemoryMap<&'static [u8]>> {
    /// Create a connector using virtual address mappings
    ///
    /// # Safety
    ///
    /// This connector assumes the memory map is valid. Failure for this condition to be met leads
    /// to undefined behaviour (most likely a segfault) when reading.
    pub unsafe fn from_addrmap(map: MemoryMap<(Address, usize)>) -> Self {
        let mut ret_map = MemoryMap::new();

        map.into_iter()
            .map(|(base, (real_base, size))| {
                (
                    base,
                    std::slice::from_raw_parts(real_base.as_u64() as _, size),
                )
            })
            .for_each(|(base, buf)| {
                ret_map.push(base, buf);
            });

        Self::with_info(ret_map)
        //Self::with_info(map.into_bufmap::<'static>())
    }
}

impl<T: AsRef<[u8]>, F: AsRef<MemoryMap<T>>> MappedPhysicalMemory<T, F> {
    pub fn with_info(info: F) -> Self {
        Self {
            info,
            marker: Default::default(),
        }
    }
}

impl<'a, F: AsRef<MemoryMap<&'a mut [u8]>> + Send> PhysicalMemory
    for MappedPhysicalMemory<&'a mut [u8], F>
{
    fn phys_read_raw_list(&mut self, data: &mut [PhysicalReadData]) -> Result<()> {
        let mut void = FnExtend::void();
        for (mapped_buf, buf) in self.info.as_ref().map_iter(
            data.iter_mut()
                .map(|PhysicalReadData(addr, buf)| (*addr, &mut **buf)),
            &mut void,
        ) {
            buf.copy_from_slice(mapped_buf.as_ref());
        }
        Ok(())
    }

    fn phys_write_raw_list(&mut self, data: &[PhysicalWriteData]) -> Result<()> {
        let mut void = FnExtend::void();

        for (mapped_buf, buf) in self
            .info
            .as_ref()
            .map_iter(data.iter().copied().map(<_>::from), &mut void)
        {
            mapped_buf.as_mut().copy_from_slice(buf);
        }

        Ok(())
    }

    fn metadata(&self) -> PhysicalMemoryMetadata {
        PhysicalMemoryMetadata {
            size: self
                .info
                .as_ref()
                .iter()
                .last()
                .map(|map| map.base().as_usize() + map.output().len())
                .unwrap(),
            readonly: false,
        }
    }
}

impl<'a, F: AsRef<MemoryMap<&'a [u8]>> + Send> PhysicalMemory
    for MappedPhysicalMemory<&'a [u8], F>
{
    fn phys_read_raw_list(&mut self, data: &mut [PhysicalReadData]) -> Result<()> {
        let mut void = FnExtend::void();
        for (mapped_buf, buf) in self.info.as_ref().map_iter(
            data.iter_mut()
                .map(|PhysicalReadData(addr, buf)| (*addr, &mut **buf)),
            &mut void,
        ) {
            buf.copy_from_slice(mapped_buf.as_ref());
        }
        Ok(())
    }

    fn phys_write_raw_list(&mut self, _data: &[PhysicalWriteData]) -> Result<()> {
        Err(Error::Connector("Target mapping is not writeable"))
    }

    fn metadata(&self) -> PhysicalMemoryMetadata {
        PhysicalMemoryMetadata {
            size: self
                .info
                .as_ref()
                .iter()
                .last()
                .map(|map| map.base().as_usize() + map.output().len())
                .unwrap(),
            readonly: true,
        }
    }
}
