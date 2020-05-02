use log::info;

use flow_core::*;
use flow_derive::*;

use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};

const LENGTH_2GB: Length = Length::from_gb(2);

#[derive(AccessVirtualMemory)]
pub struct Memory {
    pub pid: i32,
    pub map: procfs::process::MemoryMap,
    file: File,
}

impl Clone for Memory {
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
        }
    }
}

impl Memory {
    pub fn new() -> Result<Self> {
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
        })
    }
}

// TODO: evaluate use of memmap
impl AccessPhysicalMemory for Memory {
    fn phys_read_raw_into(
        &mut self,
        addr: Address,
        _page_type: PageType,
        out: &mut [u8],
    ) -> Result<()> {
        let ofs = self.map.address.0 + {
            if addr.as_u64() <= LENGTH_2GB.as_u64() {
                addr.as_u64()
            } else {
                addr.as_u64() - LENGTH_2GB.as_u64()
            }
        };
        self.file.seek(SeekFrom::Start(ofs))?;
        let _ = self.file.read(out);
        Ok(())
    }

    fn phys_write_raw(&mut self, addr: Address, _page_type: PageType, data: &[u8]) -> Result<()> {
        let ofs = self.map.address.0 + {
            if addr.as_u64() <= LENGTH_2GB.as_u64() {
                addr.as_u64()
            } else {
                addr.as_u64() - LENGTH_2GB.as_u64()
            }
        };
        self.file.seek(SeekFrom::Start(ofs))?;
        let _ = self.file.write(data);
        Ok(())
    }
}
