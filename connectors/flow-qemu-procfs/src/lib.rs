use log::info;

use flow_core::*;
use flow_derive::*;

use core::ffi::c_void;
use libc::{c_ulong, iovec, pid_t, sysconf, _SC_IOV_MAX};

const LENGTH_2GB: Length = Length::from_gb(2);

#[derive(AccessVirtualMemory, VirtualAddressTranslator)]
pub struct Memory {
    pub pid: pid_t,
    pub map: procfs::process::MemoryMap,
    iov_max: usize,
    temp_iov: Vec<iovec>,
}

impl Clone for Memory {
    fn clone(&self) -> Self {
        Self {
            pid: self.pid,
            map: self.map.clone(),
            iov_max: self.iov_max,
            temp_iov: Vec::with_capacity(self.iov_max * 2),
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

        let iov_max = unsafe { sysconf(_SC_IOV_MAX) } as usize;

        Ok(Self {
            pid: prc.stat.pid,
            map: map.clone(),
            iov_max,
            temp_iov: Vec::with_capacity(iov_max * 2),
        })
    }
}

// TODO: evaluate use of memmap
impl AccessPhysicalMemory for Memory {
    fn phys_read_raw_into(&mut self, addr: PhysicalAddress, out: &mut [u8]) -> Result<()> {
        let ofs = self.map.address.0 + {
            if addr.as_u64() <= LENGTH_2GB.as_u64() {
                addr.as_u64()
            } else {
                addr.as_u64() - LENGTH_2GB.as_u64()
            }
        };

        process_vm_read(self.pid, &[(ofs, out)], &mut self.temp_iov, self.iov_max)
    }

    fn phys_write_raw(&mut self, addr: PhysicalAddress, data: &[u8]) -> Result<()> {
        let ofs = self.map.address.0 + {
            if addr.as_u64() <= LENGTH_2GB.as_u64() {
                addr.as_u64()
            } else {
                addr.as_u64() - LENGTH_2GB.as_u64()
            }
        };

        process_vm_write(self.pid, &[(ofs, data)], &mut self.temp_iov, self.iov_max)
    }
}

fn process_vm_rw<F, T: std::convert::AsRef<[F]>>(
    pid: pid_t,
    data: &[(u64, T)],
    temp_iov: &mut Vec<iovec>,
    iov_max: usize,
    write: bool,
) -> Result<()> {
    let process_vm_rw_func = if write {
        libc::process_vm_writev
    } else {
        libc::process_vm_readv
    };

    for data in data.chunks(iov_max) {
        temp_iov.clear();

        for (_, i) in data.iter() {
            temp_iov.push(iovec {
                iov_base: i.as_ref().as_ptr() as *mut c_void,
                iov_len: i.as_ref().len(),
            });
        }

        for (addr, i) in data.iter() {
            temp_iov.push(iovec {
                iov_base: *addr as *mut c_void,
                iov_len: i.as_ref().len(),
            });
        }

        let ret = unsafe {
            process_vm_rw_func(
                pid,
                temp_iov.as_ptr(),
                data.len() as c_ulong,
                temp_iov.as_ptr().add(data.len()),
                data.len() as c_ulong,
                0,
            )
        };

        if ret == -1 {
            return Err(flow_core::error::Error::new("process_vm_rw failed"));
        }
    }

    Ok(())
}

fn process_vm_read(
    pid: pid_t,
    data: &[(u64, &mut [u8])],
    temp_iov: &mut Vec<iovec>,
    iov_max: usize,
) -> Result<()> {
    process_vm_rw(pid, data, temp_iov, iov_max, false)
}

fn process_vm_write(
    pid: pid_t,
    data: &[(u64, &[u8])],
    temp_iov: &mut Vec<iovec>,
    iov_max: usize,
) -> Result<()> {
    process_vm_rw(pid, data, temp_iov, iov_max, true)
}
