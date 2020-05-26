use log::info;

use flow_core::iter::FlowIters;
use flow_core::types::{Done, Progress, ToDo};
use flow_core::*;
use flow_derive::*;

use core::ffi::c_void;
use libc::{c_ulong, iovec, pid_t, sysconf, _SC_IOV_MAX};

use std::collections::VecDeque as VecType;

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

    pub fn filter_element<A, B>(
        elem: Progress<A, B>,
        cnt: &mut usize,
        iov_max: usize,
    ) -> (bool, Progress<A, B>) {
        (
            if let ToDo(_) = elem {
                *cnt += 1;
                (*cnt % iov_max) == 0
            } else {
                true
            },
            elem,
        )
    }

    pub fn perform_rw<A2: AsRef<[A3]>, A3>(
        &mut self,
        vec_in: &mut VecType<Progress<(PhysicalAddress, A2), Result<(PhysicalAddress, A2)>>>,
        vec_out: &mut VecType<Progress<(PhysicalAddress, A2), Result<(PhysicalAddress, A2)>>>,
        process_vm_rw_func: unsafe extern "C" fn(
            pid_t,
            *const iovec,
            c_ulong,
            *const iovec,
            c_ulong,
            c_ulong,
        ) -> isize,
    ) {
        let mut data_cnt = 0;
        let max_iov = self.temp_iov.len() / 2;

        let (iov_local, iov_remote) = self.temp_iov.split_at_mut(max_iov);

        let iov_iter = iov_local.iter_mut().zip(iov_remote.iter_mut());

        for (item, (liov, riov)) in vec_in.into_iter().zip(iov_iter) {
            if let ToDo((addr, out)) = item {
                data_cnt += 1;
                *liov = iovec {
                    iov_base: out.as_ref().as_ptr() as *mut c_void,
                    iov_len: out.as_ref().len(),
                };

                let ofs = self.map.address.0 + {
                    if addr.as_u64() <= LENGTH_2GB.as_u64() {
                        addr.as_u64()
                    } else {
                        addr.as_u64() - LENGTH_2GB.as_u64()
                    }
                };

                *riov = iovec {
                    iov_base: ofs as *mut c_void,
                    iov_len: out.as_ref().len(),
                };
            }
        }

        let ret = unsafe {
            process_vm_rw_func(
                self.pid,
                self.temp_iov.as_ptr(),
                data_cnt as c_ulong,
                self.temp_iov.as_ptr().add(max_iov),
                data_cnt as c_ulong,
                0,
            )
        };

        while let Some(item) = vec_in.pop_front() {
            vec_out.push_back(match item {
                ToDo((addr, out)) => Done(match ret {
                    -1 => Err(Error::new("process_vm_rw failed")),
                    _ => Ok((addr, out)),
                }),
                _ => item,
            });
        }
    }
}

impl AccessPhysicalMemory for Memory {
    fn phys_read_raw_iter<'a, PI: PhysicalReadIterator<'a>>(
        &'a mut self,
        iter: PI,
    ) -> Box<dyn PhysicalReadIterator<'a>> {
        let iov_max = self.temp_iov.len() / 2;
        let mut cnt = 0;

        let iter = iter.double_peekable();

        if !iter.is_next_last() {
            Box::new(
                iter.double_buffered_map(
                    move |x| Self::filter_element(x, &mut cnt, iov_max),
                    move |vec_in, vec_out| self.perform_rw(vec_in, vec_out, libc::process_vm_readv),
                    )
                )
        } else {
            //Non-batcvhing impl.
            Box::new(
                iter
                .map(move |x| match x {
                    ToDo((addr, out)) => {
                        let ofs = self.map.address.0 + {
                            if addr.as_u64() <= LENGTH_2GB.as_u64() {
                                addr.as_u64()
                            } else {
                                addr.as_u64() - LENGTH_2GB.as_u64()
                            }
                        };

                        self.temp_iov[1] = iovec {
                            iov_base: ofs as *mut c_void,
                            iov_len: out.as_ref().len(),
                        };

                        self.temp_iov[0] = iovec {
                            iov_base: out.as_ref().as_ptr() as *mut c_void,
                            iov_len: out.as_ref().len(),
                        };

                        Done(match unsafe {
                            libc::process_vm_readv(self.pid, self.temp_iov.as_ptr(), 1, self.temp_iov.as_ptr().offset(1), 1, 0)
                        } {
                            -1 => Err(Error::new("process_vm_readv failed")),
                            _ => Ok((addr, out))
                        })
                    },
                    _ => x
                })
            )
        }
    }

    fn phys_write_raw_iter<'a, PI: PhysicalWriteIterator<'a>>(
        &'a mut self,
        iter: PI,
    ) -> Box<dyn PhysicalWriteIterator<'a>> {
        let iov_max = self.temp_iov.len() / 2;
        let mut cnt = 0;
        Box::new(iter.double_buffered_map(
            move |x| Self::filter_element(x, &mut cnt, iov_max),
            move |vec_in, vec_out| self.perform_rw(vec_in, vec_out, libc::process_vm_writev),
        ))
    }
}
