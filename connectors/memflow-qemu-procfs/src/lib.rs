use log::info;

use memflow_core::connector::ConnectorArgs;
use memflow_core::*;
use memflow_derive::connector;

use core::ffi::c_void;
use libc::{c_ulong, iovec, pid_t, sysconf, _SC_IOV_MAX};

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
pub struct QemuProcfs {
    pub pid: pid_t,
    pub mem_map: MemoryMap<(Address, usize)>,
    temp_iov: Box<[iovec]>,
}

impl QemuProcfs {
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
        // find biggest memory mapping in qemu process
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

        let map_base = map.address.0 as usize;
        let map_size = (map.address.1 - map.address.0) as usize;
        info!("qemu memory map size: {:x}", map_size);

        // TODO: instead of hardcoding the memory regions per machine we could just use the hmp to retrieve the proper ranges:
        // sudo virsh qemu-monitor-command win10 --hmp 'info mtree -f' | grep pc\.ram

        // find machine architecture
        let machine = qemu_arg_opt(
            &prc.cmdline()
                .map_err(|_| Error::Connector("unable to parse qemu arguments"))?,
            "-machine",
            "type",
        )
        .unwrap_or_else(|| "pc".into());
        info!("qemu process started with machine: {}", machine);

        let mut mem_map = MemoryMap::new();
        if machine.contains("q35") {
            // q35 -> subtract 2GB
            /*
            0000000000000000-000000000009ffff (prio 0, ram): pc.ram KVM
            00000000000c0000-00000000000c3fff (prio 0, rom): pc.ram @00000000000c0000 KVM
            0000000000100000-000000007fffffff (prio 0, ram): pc.ram @0000000000100000 KVM
            0000000100000000-000000047fffffff (prio 0, ram): pc.ram @0000000080000000 KVM
            */
            // we add all regions additionally shifted to the proper qemu memory map address
            mem_map.push_range(Address::NULL, size::kb(640).into(), map_base.into()); // section: [start - 640kb] -> map to start
            mem_map.push_range(
                size::mb(1).into(),
                size::gb(2).into(),
                (map_base + size::mb(1)).into(),
            ); // section: [1mb - 2gb] -> map to 1mb
            mem_map.push_range(
                size::gb(4).into(),
                (map_size + size::gb(2)).into(),
                (map_base + size::gb(2)).into(),
            ); // section: [4gb - max] -> map to 2gb
        } else {
            // pc-i1440fx
            /*
            0000000000000000-00000000000bffff (prio 0, ram): pc.ram KVM
            00000000000c0000-00000000000cafff (prio 0, rom): pc.ram @00000000000c0000 KVM
            00000000000cb000-00000000000cdfff (prio 0, ram): pc.ram @00000000000cb000 KVM
            00000000000ce000-00000000000e7fff (prio 0, rom): pc.ram @00000000000ce000 KVM
            00000000000e8000-00000000000effff (prio 0, ram): pc.ram @00000000000e8000 KVM
            00000000000f0000-00000000000fffff (prio 0, rom): pc.ram @00000000000f0000 KVM
            0000000000100000-00000000bfffffff (prio 0, ram): pc.ram @0000000000100000 KVM
            0000000100000000-000000023fffffff (prio 0, ram): pc.ram @00000000c0000000 KVM
            */
            mem_map.push_range(Address::NULL, size::kb(768).into(), map_base.into()); // section: [start - 768kb] -> map to start
            mem_map.push_range(
                size::kb(812).into(),
                size::kb(824).into(),
                (map_base + size::kb(812)).into(),
            ); // section: [768kb - 812kb] -> map to 768kb
            mem_map.push_range(
                size::kb(928).into(),
                size::kb(960).into(),
                (map_base + size::kb(928)).into(),
            ); // section: [928kb - 960kb] -> map to 928kb
            mem_map.push_range(
                size::mb(1).into(),
                size::gb(3).into(),
                (map_base + size::mb(1)).into(),
            ); // section: [1mb - 3gb] -> map to 1mb
            mem_map.push_range(
                size::gb(4).into(),
                (map_size + size::gb(1)).into(),
                (map_base + size::gb(3)).into(),
            ); // section: [4gb - max] -> map to 3gb
        }
        info!("qemu machine mem_map: {:?}", mem_map);

        let iov_max = unsafe { sysconf(_SC_IOV_MAX) } as usize;

        Ok(Self {
            pid: prc.stat.pid,
            mem_map,
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

    fn fill_iovec(addr: &Address, data: &[u8], liov: &mut iovec, riov: &mut iovec) {
        let iov_len = data.len();

        *liov = iovec {
            iov_base: data.as_ptr() as *mut c_void,
            iov_len,
        };

        *riov = iovec {
            iov_base: addr.as_u64() as *mut c_void,
            iov_len,
        };
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

impl PhysicalMemory for QemuProcfs {
    fn phys_read_raw_list(&mut self, data: &mut [PhysicalReadData]) -> Result<()> {
        let mem_map = &self.mem_map;
        let temp_iov = &mut self.temp_iov;

        let mut void = FnExtend::void();
        let mut iter = mem_map.map_iter(
            data.iter_mut().map(|(addr, buf)| (*addr, &mut **buf)),
            &mut void,
        );

        let max_iov = temp_iov.len() / 2;
        let (iov_local, iov_remote) = temp_iov.split_at_mut(max_iov);

        let mut elem = iter.next();

        let mut iov_iter = iov_local.iter_mut().zip(iov_remote.iter_mut()).enumerate();
        let mut iov_next = iov_iter.next();

        while let Some(((addr, _), out)) = elem {
            let (cnt, (liov, riov)) = iov_next.unwrap();

            Self::fill_iovec(&addr, out, liov, riov);

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

    fn phys_write_raw_list(&mut self, data: &[PhysicalWriteData]) -> Result<()> {
        let mem_map = &self.mem_map;
        let temp_iov = &mut self.temp_iov;

        let mut void = FnExtend::void();
        let mut iter = mem_map.map_iter(data.iter().copied(), &mut void);
        //let mut iter = mem_map.map_iter(data.iter(), &mut FnExtend::new(|_|{}));

        let max_iov = temp_iov.len() / 2;
        let (iov_local, iov_remote) = temp_iov.split_at_mut(max_iov);

        let mut elem = iter.next();

        let mut iov_iter = iov_local.iter_mut().zip(iov_remote.iter_mut()).enumerate();
        let mut iov_next = iov_iter.next();

        while let Some(((addr, _), out)) = elem {
            let (cnt, (liov, riov)) = iov_next.unwrap();

            Self::fill_iovec(&addr, out, liov, riov);

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

// TODO: handle args properly
/// Creates a new Qemu Procfs Connector instance.
#[connector(name = "qemu_procfs")]
pub fn create_connector(args: &ConnectorArgs) -> Result<QemuProcfs> {
    if let Some(name) = args.get("name").or_else(|| args.get_default()) {
        QemuProcfs::with_guest_name(name)
    } else {
        QemuProcfs::new()
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
