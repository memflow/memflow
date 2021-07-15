/*!
Basic connector which works on mapped memory.
*/

use crate::error::{Error, ErrorKind, ErrorOrigin, Result};
use crate::mem::{
    MemData, MemoryMap, PhysicalMemory, PhysicalMemoryMapping, PhysicalMemoryMetadata,
    PhysicalReadData, PhysicalWriteData,
};
use crate::types::Address;

use crate::cglue::*;
use crate::mem::phys_mem::*;

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
                    std::slice::from_raw_parts_mut(real_base.to_umem() as _, size),
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
                    std::slice::from_raw_parts(real_base.to_umem() as _, size),
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
    fn phys_read_raw_iter<'b>(
        &mut self,
        data: CIterator<PhysicalReadData<'b>>,
        out_fail: &mut PhysicalReadFailCallback<'_, 'b>,
    ) -> Result<()> {
        let mut void = |(addr, buf): (Address, &'b mut [u8])| {
            // TODO: manage not to lose physical page information here???
            out_fail.call(MemData(addr.into(), buf.into()))
        };
        for (mapped_buf, buf) in self
            .info
            .as_ref()
            .map_iter(data.map(|MemData(addr, buf)| (addr, buf.into())), &mut void)
        {
            buf.copy_from_slice(mapped_buf.as_ref());
        }
        Ok(())
    }

    fn phys_write_raw_iter<'b>(
        &mut self,
        data: CIterator<PhysicalWriteData<'b>>,
        out_fail: &mut PhysicalWriteFailCallback<'_, 'b>,
    ) -> Result<()> {
        let mut void = &mut |(addr, buf): (Address, _)| {
            // TODO: manage not to lose physical page information here???
            out_fail.call(MemData(addr.into(), buf))
        };

        for (mapped_buf, buf) in self.info.as_ref().map_iter(data.map(<_>::from), &mut void) {
            mapped_buf.as_mut().copy_from_slice(buf.into());
        }

        Ok(())
    }

    fn metadata(&self) -> PhysicalMemoryMetadata {
        let max_address = self
            .info
            .as_ref()
            .iter()
            .last()
            .map(|map| map.base().to_umem() + map.output().len())
            .unwrap()
            - 1;
        let real_size = self
            .info
            .as_ref()
            .iter()
            .fold(0, |s, m| s + m.output().len() as u64);
        PhysicalMemoryMetadata {
            max_address: max_address.into(),
            real_size,
            readonly: false,
            ideal_batch_size: u32::MAX,
        }
    }

    // This is a no-op for u8 slices.
    fn set_mem_map(&mut self, _mem_map: &[PhysicalMemoryMapping]) {}
}

impl<'a, F: AsRef<MemoryMap<&'a [u8]>> + Send> PhysicalMemory
    for MappedPhysicalMemory<&'a [u8], F>
{
    fn phys_read_raw_iter<'b>(
        &mut self,
        data: CIterator<PhysicalReadData<'b>>,
        out_fail: &mut PhysicalReadFailCallback<'_, 'b>,
    ) -> Result<()> {
        let mut void = |(addr, buf): (Address, _)| {
            // TODO: manage not to lose physical page information here???
            out_fail.call(MemData(addr.into(), buf))
        };
        for (mapped_buf, mut buf) in self.info.as_ref().map_iter(data.map(<_>::into), &mut void) {
            buf.copy_from_slice(mapped_buf.as_ref());
        }
        Ok(())
    }

    fn phys_write_raw_iter<'b>(
        &mut self,
        _data: CIterator<PhysicalWriteData<'b>>,
        _out_fail: &mut PhysicalWriteFailCallback<'_, 'b>,
    ) -> Result<()> {
        Err(Error(ErrorOrigin::Connector, ErrorKind::ReadOnly)
            .log_error("target mapping is not writeable"))
    }

    fn metadata(&self) -> PhysicalMemoryMetadata {
        let max_address = self
            .info
            .as_ref()
            .iter()
            .last()
            .map(|map| map.base().to_umem() + map.output().len())
            .unwrap()
            - 1;
        let real_size = self
            .info
            .as_ref()
            .iter()
            .fold(0, |s, m| s + m.output().len() as u64);
        PhysicalMemoryMetadata {
            max_address: max_address.into(),
            real_size,
            readonly: true,
            ideal_batch_size: u32::MAX,
        }
    }

    // This is a no-op for u8 slices.
    fn set_mem_map(&mut self, _mem_map: &[PhysicalMemoryMapping]) {}
}

#[cfg(feature = "plugins")]
cglue_impl_group!(
    MappedPhysicalMemory<T = &'cglue_a mut [u8], F: AsRef<MemoryMap<&'cglue_a mut [u8]>>>,
    ConnectorInstance,
    {}
);
#[cfg(feature = "plugins")]
cglue_impl_group!(
    MappedPhysicalMemory<T = &'cglue_a [u8], F: AsRef<MemoryMap<&'cglue_a [u8]>>>,
    ConnectorInstance,
    {}
);
