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
    temp_iov: Box<[iovec]>,
}

impl Clone for Memory {
    fn clone(&self) -> Self {
        Self {
            pid: self.pid,
            map: self.map.clone(),
            temp_iov: self.temp_iov.clone(),
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
            temp_iov: vec![
                iovec {
                    iov_base: std::ptr::null_mut::<c_void>(),
                    iov_len: 0
                };
                iov_max * 2
            ]
            .into_boxed_slice(),
        })
    }

    fn process_vm_rw<F, T: std::convert::AsRef<[F]>>(
        &mut self,
        data: &[(PhysicalAddress, T)],
        write: bool,
    ) -> Result<()> {
        let process_vm_rw_func = if write {
            libc::process_vm_writev
        } else {
            libc::process_vm_readv
        };

        let iov_max = self.temp_iov.len() / 2;

        for data in data.chunks(iov_max) {
            for ((_, i), &mut ref mut iov) in data.iter().zip(self.temp_iov.iter_mut()) {
                *iov = iovec {
                    iov_base: i.as_ref().as_ptr() as *mut c_void,
                    iov_len: i.as_ref().len(),
                };
            }

            for ((addr, i), &mut ref mut iov) in
                data.iter().zip(self.temp_iov[data.len()..].iter_mut())
            {
                let ofs = self.map.address.0 + {
                    if addr.as_u64() <= LENGTH_2GB.as_u64() {
                        addr.as_u64()
                    } else {
                        addr.as_u64() - LENGTH_2GB.as_u64()
                    }
                };

                *iov = iovec {
                    iov_base: ofs as *mut c_void,
                    iov_len: i.as_ref().len(),
                };
            }

            let ret = unsafe {
                process_vm_rw_func(
                    self.pid,
                    self.temp_iov.as_ptr(),
                    data.len() as c_ulong,
                    self.temp_iov.as_ptr().add(data.len()),
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
}

impl AccessPhysicalMemory for Memory {
    fn phys_read_raw_into(&mut self, addr: PhysicalAddress, out: &mut [u8]) -> Result<()> {
        self.process_vm_rw(&[(addr, out)], false)
    }

    fn phys_write_raw(&mut self, addr: PhysicalAddress, data: &[u8]) -> Result<()> {
        self.process_vm_rw(&[(addr, data)], true)
    }
}
