use log::info;

use flow_core::*;

use core::ffi::c_void;
use libc::{c_ulong, iovec, pid_t, sysconf, _SC_IOV_MAX};

const SIZE_2GB: usize = size::gb(2);
const SIZE_1GB: usize = size::gb(1);

fn qemu_arg_opt(args: &[String], argname: &str, argopt: &str) -> Option<String> {
    for (idx, arg) in args.iter().enumerate() {
        if arg == argname {
            let name = args[idx + 1].split(',');
            for (i, kv) in name.clone().enumerate() {
                let kvsplt = kv.split('=').collect::<Vec<_>>();
                if kvsplt.len() == 2 {
                    if kvsplt[0] == argopt {
                        return Some(kvsplt[1].to_string());
                    }
                } else if i == 0 {
                    return Some(kv.to_string());
                }
            }
        }
    }

    None
}

#[derive(Clone)]
pub struct Memory {
    pub pid: pid_t,
    pub map: procfs::process::MemoryMap,
    pub hw_offset: usize,
    temp_iov: Box<[iovec]>,
}

impl Memory {
    pub fn new() -> Result<Self> {
        let prcs = procfs::process::all_processes()
            .map_err(|_| Error::Connector("unable to list procfs processes"))?;
        let prc = prcs
            .iter()
            .find(|p| p.stat.comm == "qemu-system-x86")
            .ok_or_else(|| Error::Connector("qemu process not found"))?;
        info!("qemu process found with pid {:?}", prc.stat.pid);

        Self::with_process(prc)
    }

    pub fn with_guest_name(name: &str) -> Result<Self> {
        let prcs = procfs::process::all_processes()
            .map_err(|_| Error::Connector("unable to list procefs processes"))?;
        let (prc, _) = prcs
            .iter()
            .filter(|p| p.stat.comm == "qemu-system-x86")
            .filter_map(|p| {
                if let Ok(c) = p.cmdline() {
                    Some((p, c))
                } else {
                    None
                }
            })
            .find(|(_, c)| qemu_arg_opt(c, "-name", "guest").unwrap_or_default() == name)
            .ok_or_else(|| Error::Connector("qemu process not found"))?;
        info!(
            "qemu process with name {} found with pid {:?}",
            name, prc.stat.pid
        );

        Self::with_process(prc)
    }

    fn with_process(prc: &procfs::process::Process) -> Result<Self> {
        // find machine architecture
        let machine = qemu_arg_opt(
            &prc.cmdline()
                .map_err(|_| Error::Connector("unable to parse qemu arguments"))?,
            "-machine",
            "type",
        )
        .unwrap_or_else(|| "pc".into());
        info!("qemu process started with machine: {}", machine);

        // this is quite an ugly hack...
        let hw_offset = {
            if machine.contains("q35") {
                // q35 -> subtract 2GB
                SIZE_2GB
            } else {
                // pc-i1440fx -> subtract 1GB
                SIZE_1GB
            }
        };
        info!("qemu machine hardware offset: {:x}", hw_offset);

        // find biggest mapping
        let mut maps = prc
            .maps()
            .map_err(|_| Error::Connector("unable to get qemu memory maps"))?;
        maps.sort_by(|b, a| {
            (a.address.1 - a.address.0)
                .partial_cmp(&(b.address.1 - b.address.0))
                .unwrap()
        });
        let map = maps
            .get(0)
            .ok_or_else(|| Error::Connector("qemu memory map could not be read"))?;
        info!("qemu memory map found {:?}", map);

        let iov_max = unsafe { sysconf(_SC_IOV_MAX) } as usize;

        Ok(Self {
            pid: prc.stat.pid,
            map: map.clone(),
            hw_offset,
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
        map_address: (u64, u64),
        hw_offset: u64,
    ) -> bool {
        let ofs = map_address.0 + {
            if addr.as_u64() <= (SIZE_2GB as u64) {
                addr.as_u64()
            } else {
                addr.as_u64() - hw_offset
            }
        };

        let iov_len = if ofs < map_address.0 || ofs + data.len() as u64 > map_address.1 {
            0
        } else {
            data.len()
        };

        *liov = iovec {
            iov_base: data.as_ptr() as *mut c_void,
            iov_len,
        };

        *riov = iovec {
            iov_base: ofs as *mut c_void,
            iov_len,
        };

        iov_len == data.len()
    }

    fn vm_error() -> Error {
        match unsafe { *libc::__errno_location() } {
            libc::EFAULT => Error::Connector("process_vm_readv failed: EFAULT (remote memory address is invalid)"),
            libc::ENOMEM => Error::Connector("process_vm_readv failed: ENOMEM (unable to allocate memory for internal copies)"),
            libc::EPERM => Error::Connector("process_vm_readv failed: EPERM (insifficient permissions to access the target address space)"),
            libc::ESRCH => Error::Connector("process_vm_readv failed: ESRCH (process not found)"),
            libc::EINVAL => Error::Connector("process_vm_readv failed: EINVAL (invalid value)"),
            _ => Error::Connector("process_vm_readv failed: unknown error")
        }
    }
}

impl PhysicalMemory for Memory {
    fn phys_read_iter<'a, PI: PhysicalReadIterator<'a>>(&'a mut self, mut iter: PI) -> Result<()> {
        let max_iov = self.temp_iov.len() / 2;
        let (iov_local, iov_remote) = self.temp_iov.split_at_mut(max_iov);

        let mut elem = iter.next();

        let mut iov_iter = iov_local.iter_mut().zip(iov_remote.iter_mut()).enumerate();
        let mut iov_next = iov_iter.next();

        while let Some((addr, out)) = elem {
            let (cnt, (liov, riov)) = iov_next.unwrap();

            if !Self::fill_iovec(
                &addr,
                out,
                liov,
                riov,
                self.map.address,
                self.hw_offset as u64,
            ) {
                //We might want to zero out the memory here
            }

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
                    return Err(Self::vm_error());
                }

                iov_iter = iov_local.iter_mut().zip(iov_remote.iter_mut()).enumerate();
                iov_next = iov_iter.next();
            }
        }

        Ok(())
    }

    fn phys_write_iter<'a, PI: PhysicalWriteIterator<'a>>(
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

            Self::fill_iovec(
                &addr,
                out,
                liov,
                riov,
                self.map.address,
                self.hw_offset as u64,
            );

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
                    return Err(Self::vm_error());
                }

                iov_iter = iov_local.iter_mut().zip(iov_remote.iter_mut()).enumerate();
                iov_next = iov_iter.next();
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_name() {
        assert_eq!(
            qemu_arg_opt(
                &["-name".to_string(), "win10-test".to_string()],
                "-name",
                "guest"
            ),
            Some("win10-test".into())
        );
        assert_eq!(
            qemu_arg_opt(
                &[
                    "-test".to_string(),
                    "-name".to_string(),
                    "win10-test".to_string()
                ],
                "-name",
                "guest"
            ),
            Some("win10-test".into())
        );
        assert_eq!(
            qemu_arg_opt(
                &["-name".to_string(), "win10-test,arg=opt".to_string()],
                "-name",
                "guest"
            ),
            Some("win10-test".into())
        );
        assert_eq!(
            qemu_arg_opt(
                &["-name".to_string(), "guest=win10-test,arg=opt".to_string()],
                "-name",
                "guest"
            ),
            Some("win10-test".into())
        );
        assert_eq!(
            qemu_arg_opt(
                &["-name".to_string(), "arg=opt,guest=win10-test".to_string()],
                "-name",
                "guest"
            ),
            Some("win10-test".into())
        );
        assert_eq!(
            qemu_arg_opt(
                &["-name".to_string(), "arg=opt".to_string()],
                "-name",
                "guest"
            ),
            None
        );
    }

    #[test]
    fn test_machine() {
        assert_eq!(
            qemu_arg_opt(
                &["-machine".to_string(), "q35".to_string()],
                "-machine",
                "type"
            ),
            Some("q35".into())
        );
        assert_eq!(
            qemu_arg_opt(
                &[
                    "-test".to_string(),
                    "-machine".to_string(),
                    "q35".to_string()
                ],
                "-machine",
                "type"
            ),
            Some("q35".into())
        );
        assert_eq!(
            qemu_arg_opt(
                &["-machine".to_string(), "q35,arg=opt".to_string()],
                "-machine",
                "type"
            ),
            Some("q35".into())
        );
        assert_eq!(
            qemu_arg_opt(
                &["-machine".to_string(), "type=pc,arg=opt".to_string()],
                "-machine",
                "type"
            ),
            Some("pc".into())
        );
        assert_eq!(
            qemu_arg_opt(
                &[
                    "-machine".to_string(),
                    "arg=opt,type=pc-i1440fx".to_string()
                ],
                "-machine",
                "type"
            ),
            Some("pc-i1440fx".into())
        );
        assert_eq!(
            qemu_arg_opt(
                &["-machine".to_string(), "arg=opt".to_string()],
                "-machine",
                "type"
            ),
            None
        );
    }
}
