//! Basic connector which works on mapped memory.
use crate::error::{Error, ErrorKind, ErrorOrigin, Result};
use crate::mem::{
    opt_call, MemoryMap, PhysicalMemory, PhysicalMemoryMetadata, PhysicalReadMemOps,
    PhysicalWriteMemOps,
};
use crate::types::{umem, Address};

use crate::cglue::*;

use std::convert::TryInto;

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
    pub unsafe fn from_addrmap_mut(map: MemoryMap<(Address, umem)>) -> Self {
        let mut ret_map = MemoryMap::new();

        map.into_iter()
            .map(|(base, (real_base, size))| {
                (
                    base,
                    std::slice::from_raw_parts_mut(
                        real_base.to_umem() as _,
                        size.try_into().unwrap(),
                    ),
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
    pub unsafe fn from_addrmap(map: MemoryMap<(Address, umem)>) -> Self {
        let mut ret_map = MemoryMap::new();

        map.into_iter()
            .map(|(base, (real_base, size))| {
                (
                    base,
                    std::slice::from_raw_parts(real_base.to_umem() as _, size.try_into().unwrap()),
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

#[allow(clippy::needless_option_as_deref)]
impl<'a, F: AsRef<MemoryMap<&'a mut [u8]>> + Send> PhysicalMemory
    for MappedPhysicalMemory<&'a mut [u8], F>
{
    fn phys_read_raw_iter(&mut self, mut data: PhysicalReadMemOps) -> Result<()> {
        for CTup3(mapped_buf, meta_addr, mut buf) in
            self.info.as_ref().map_iter(data.inp, data.out_fail)
        {
            buf.copy_from_slice(mapped_buf.as_ref());
            opt_call(data.out.as_deref_mut(), CTup2(meta_addr, buf));
        }
        Ok(())
    }

    fn phys_write_raw_iter(&mut self, mut data: PhysicalWriteMemOps) -> Result<()> {
        for CTup3(mapped_buf, meta_addr, buf) in
            self.info.as_ref().map_iter(data.inp, data.out_fail)
        {
            mapped_buf.as_mut().copy_from_slice(buf.into());
            opt_call(data.out.as_deref_mut(), CTup2(meta_addr, buf));
        }

        Ok(())
    }

    fn metadata(&self) -> PhysicalMemoryMetadata {
        let max_address = self
            .info
            .as_ref()
            .iter()
            .last()
            .map(|map| map.base().to_umem() + map.output().len() as umem)
            .unwrap()
            - 1;
        let real_size = self
            .info
            .as_ref()
            .iter()
            .fold(0, |s, m| s + m.output().len() as umem);
        PhysicalMemoryMetadata {
            max_address: max_address.into(),
            real_size,
            readonly: false,
            ideal_batch_size: u32::MAX,
        }
    }
}

#[allow(clippy::needless_option_as_deref)]
impl<'a, F: AsRef<MemoryMap<&'a [u8]>> + Send> PhysicalMemory
    for MappedPhysicalMemory<&'a [u8], F>
{
    fn phys_read_raw_iter(&mut self, mut data: PhysicalReadMemOps) -> Result<()> {
        for CTup3(mapped_buf, meta_addr, mut buf) in
            self.info.as_ref().map_iter(data.inp, data.out_fail)
        {
            buf.copy_from_slice(mapped_buf.as_ref());
            opt_call(data.out.as_deref_mut(), CTup2(meta_addr, buf));
        }
        Ok(())
    }

    fn phys_write_raw_iter(&mut self, _data: PhysicalWriteMemOps) -> Result<()> {
        Err(Error(ErrorOrigin::Connector, ErrorKind::ReadOnly)
            .log_error("target mapping is not writeable"))
    }

    fn metadata(&self) -> PhysicalMemoryMetadata {
        let max_address = self
            .info
            .as_ref()
            .iter()
            .last()
            .map(|map| map.base().to_umem() + map.output().len() as umem)
            .unwrap()
            - 1;
        let real_size = self
            .info
            .as_ref()
            .iter()
            .fold(0, |s, m| s + m.output().len() as umem);
        PhysicalMemoryMetadata {
            max_address: max_address.into(),
            real_size,
            readonly: true,
            ideal_batch_size: u32::MAX,
        }
    }
}

#[cfg(feature = "plugins")]
cglue_impl_group!(
    MappedPhysicalMemory<T = &'cglue_a mut [u8], F: AsRef<MemoryMap<&'cglue_a mut [u8]>>>,
    crate::plugins::ConnectorInstance,
    {}
);
#[cfg(feature = "plugins")]
cglue_impl_group!(
    MappedPhysicalMemory<T = &'cglue_a [u8], F: AsRef<MemoryMap<&'cglue_a [u8]>>>,
    crate::plugins::ConnectorInstance,
    {}
);
