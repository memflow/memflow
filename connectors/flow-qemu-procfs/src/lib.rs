use log::info;

use flow_core::*;
use flow_derive::*;

use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};

const LENGTH_2GB: Length = Length::from_gb(2);

#[derive(AccessVirtualMemory)]
pub struct Memory<T: mem::PageCache + Clone> {
    pub pid: i32,
    pub map: procfs::process::MemoryMap,
    file: File,
    cache: T,
}

impl<T: mem::PageCache + Clone> Clone for Memory<T> {
    fn clone(&self) -> Self {
        let new_file = OpenOptions::new()
            .read(true)
            .write(true)
            .open(format!("/proc/{}/mem", self.pid))
            .unwrap(); // TODO: might panic

        Self {
            pid: self.pid,
            map: self.map.clone(),
            file: new_file,
            cache: self.cache.clone(),
        }
    }
}

impl<T: mem::PageCache + Clone> Memory<T> {
    pub fn new(cache: T) -> Result<Self> {
        let prcs = procfs::process::all_processes().map_err(Error::new)?;
        let prc = prcs
            .iter()
            .find(|p| p.stat.comm == "qemu-system-x86")
            .ok_or_else(|| Error::new("qemu process not found"))?;
        info!("qemu process found {:?}", prc.stat);

        // find biggest mapping
        let mut maps = prc.maps().map_err(Error::new)?;
        maps.sort_by(|b, a| {
            (a.address.1 - a.address.0)
                .partial_cmp(&(b.address.1 - b.address.0))
                .unwrap()
        });
        let map = maps
            .get(0)
            .ok_or_else(|| Error::new("qemu memory map could not be read"))?;
        info!("qemu memory map found {:?}", map);

        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .open(format!("/proc/{}/mem", prc.stat.pid))?;
        Ok(Self {
            pid: prc.stat.pid,
            map: map.clone(),
            file,
            cache,
        })
    }
}

// TODO: evaluate use of memmap
impl<T: mem::PageCache + Clone> AccessPhysicalMemory for Memory<T> {
    fn phys_read_raw_into(
        &mut self,
        addr: Address,
        page_type: mem::PageType,
        out: &mut [u8],
    ) -> Result<()> {
        let ofs = {
            if addr.as_u64() <= LENGTH_2GB.as_u64() {
                0
            } else {
                LENGTH_2GB.as_u64()
            }
        };

        let file = &mut self.file;
        let map_address = self.map.address.0;

        self.cache.cached_read(
            addr,
            page_type,
            out.as_mut(),
            |start: Address, cur_out: &mut [u8]| -> Result<()> {
                file.seek(SeekFrom::Start(map_address + start.as_u64() - ofs))?;
                let _ = file.read(cur_out);
                Ok(())
            },
        )?;

        Ok(())
    }

    fn phys_write_raw(
        &mut self,
        addr: Address,
        page_type: mem::PageType,
        data: &[u8],
    ) -> Result<()> {
        let ofs = self.map.address.0 + {
            if addr.as_u64() <= LENGTH_2GB.as_u64() {
                addr.as_u64()
            } else {
                addr.as_u64() - LENGTH_2GB.as_u64()
            }
        };
        self.file.seek(SeekFrom::Start(ofs))?;

        let _ = self.file.write(data);
        self.cache.invalidate_pages(addr, page_type, data);

        Ok(())
    }
}
