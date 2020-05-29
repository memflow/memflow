use log::info;

use flow_core::*;
use flow_derive::*;

use core::ffi::c_void;
use libc::{c_ulong, iovec, pid_t, sysconf, _SC_IOV_MAX};

const LENGTH_2GB: Length = Length::from_gb(2);

#[derive(Clone, AccessVirtualMemory, VirtualAddressTranslator)]
pub struct Memory {
    pub pid: pid_t,
    pub map: procfs::process::MemoryMap,
    temp_iov: Box<[iovec]>,
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

    pub fn fill_iovec(
        addr: &PhysicalAddress,
        data: &[u8],
        liov: &mut iovec,
        riov: &mut iovec,
        map_address: u64,
    ) {
        let ofs = map_address + {
            if addr.as_u64() <= LENGTH_2GB.as_u64() {
                addr.as_u64()
            } else {
                addr.as_u64() - LENGTH_2GB.as_u64()
            }
        };

        *liov = iovec {
            iov_base: data.as_ptr() as *mut c_void,
            iov_len: data.len(),
        };

        *riov = iovec {
            iov_base: ofs as *mut c_void,
            iov_len: data.len(),
        };
    }
}

impl AccessPhysicalMemory for Memory {
    fn phys_read_raw_iter<'a, PI: PhysicalReadIterator<'a>>(
        &'a mut self,
        mut iter: PI,
    ) -> Result<()> {
        let max_iov = self.temp_iov.len() / 2;
        let (iov_local, iov_remote) = self.temp_iov.split_at_mut(max_iov);

        let mut elem = iter.next();

        let mut iov_iter = iov_local.iter_mut().zip(iov_remote.iter_mut()).enumerate();
        let mut iov_next = iov_iter.next();

        while let Some((addr, out)) = elem {
            let (cnt, (liov, riov)) = iov_next.unwrap();

            Self::fill_iovec(&addr, out.as_ref(), liov, riov, self.map.address.0);

            iov_next = iov_iter.next();
            elem = iter.next();

            if elem.is_none() || iov_next.is_none() {
                if unsafe {
                    libc::process_vm_readv(
                        self.pid,
                        iov_local.as_ptr(),
                        (cnt + 1) as c_ulong,
                        iov_remote.as_ptr(),
                        (cnt + 1) as c_ulong,
                        0,
                    )
                } == -1
                {
                    return Err(Error::new("process_vm_readv failed"));
                }

                iov_iter = iov_local.iter_mut().zip(iov_remote.iter_mut()).enumerate();
                iov_next = iov_iter.next();
            }
        }

        Ok(())
    }

    fn phys_write_raw_iter<'a, PI: PhysicalWriteIterator<'a>>(
        &'a mut self,
        mut iter: PI,
    ) -> Result<()> {
        let max_iov = self.temp_iov.len() / 2;
        let (iov_local, iov_remote) = self.temp_iov.split_at_mut(max_iov);

        let mut elem = iter.next();

        let mut iov_iter = iov_local.iter_mut().zip(iov_remote.iter_mut()).enumerate();
        let mut iov_next = iov_iter.next();

        while let Some((addr, out)) = elem {
            let (cnt, (liov, riov)) = iov_next.unwrap();

            Self::fill_iovec(&addr, out.as_ref(), liov, riov, self.map.address.0);

            iov_next = iov_iter.next();
            elem = iter.next();

            if elem.is_none() || iov_next.is_none() {
                if unsafe {
                    libc::process_vm_writev(
                        self.pid,
                        iov_local.as_ptr(),
                        (cnt + 1) as c_ulong,
                        iov_remote.as_ptr(),
                        (cnt + 1) as c_ulong,
                        0,
                    )
                } == -1
                {
                    return Err(Error::new("process_vm_writev failed"));
                }

                iov_iter = iov_local.iter_mut().zip(iov_remote.iter_mut()).enumerate();
                iov_next = iov_iter.next();
            }
        }

        Ok(())
    }
}
