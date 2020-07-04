use flow_core::iter::{ExtendVoid, SplitAtIndexNoMutation};
use flow_core::mem::{MemoryMap, PhysicalMemory, PhysicalReadData, PhysicalWriteData};
use flow_core::types::Address;
use flow_core::{Error, Result};

pub struct MMAPInfo<T: AsRef<[u8]>> {
    mem_map: MemoryMap<T>,
}

pub struct MappedPhysicalMemory<T: AsRef<[u8]>> {
    info: MMAPInfo<T>,
}

impl MappedPhysicalMemory<&'static mut [u8]> {
    /// Create a connector using virtual address mappings
    ///
    /// Safety
    ///
    /// This connector
    pub unsafe fn from_addrmap(map: MemoryMap<(Address, usize)>) -> Self {
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

        Self::with_info(MMAPInfo { mem_map: ret_map })
    }
}

impl MappedPhysicalMemory<&'static [u8]> {
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

        Self::with_info(MMAPInfo { mem_map: ret_map })
    }
}

impl<T: AsRef<[u8]> + SplitAtIndexNoMutation> MappedPhysicalMemory<T> {
    pub fn with_info(info: MMAPInfo<T>) -> Self {
        Self { info }
    }

    pub fn try_with_bufmap(map: MemoryMap<(Address, usize)>, buf: T) -> Result<Self> {
        Err(Error::Connector("Not implemented"))
    }
}

impl<'a> PhysicalMemory for MappedPhysicalMemory<&'a mut [u8]> {
    fn phys_read_raw_list(&mut self, data: &mut [PhysicalReadData]) -> Result<()> {
        let mut void = ExtendVoid::void();
        for (mapped_buf, buf) in self.info.mem_map.map_iter(
            data.iter_mut().map(|(addr, buf)| (*addr, &mut **buf)),
            &mut void,
        ) {
            buf.copy_from_slice(mapped_buf);
        }
        Ok(())
    }

    fn phys_write_raw_list(&mut self, data: &[PhysicalWriteData]) -> Result<()> {
        let mut void = ExtendVoid::void();
        for (mapped_buf, buf) in self.info.mem_map.map_iter(data.iter().copied(), &mut void) {
            mapped_buf.copy_from_slice(buf);
        }

        Ok(())
    }
}

impl<'a> PhysicalMemory for MappedPhysicalMemory<&'a [u8]> {
    fn phys_read_raw_list(&mut self, data: &mut [PhysicalReadData]) -> Result<()> {
        let mut void = ExtendVoid::void();
        for (mapped_buf, buf) in self.info.mem_map.map_iter(
            data.iter_mut().map(|(addr, buf)| (*addr, &mut **buf)),
            &mut void,
        ) {
            buf.copy_from_slice(mapped_buf);
        }
        Ok(())
    }

    fn phys_write_raw_list(&mut self, _data: &[PhysicalWriteData]) -> Result<()> {
        Err(Error::Connector("Target mapping is not writeable"))
    }
}
