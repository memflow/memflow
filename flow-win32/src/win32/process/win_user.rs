// TODO: this module will be renamed / replaces user.rs when its finished
// this will construct a user process by itself and doesnt hold any additional data

use crate::error::{Error, Result};

use crate::offsets::Win32Offsets;
use crate::win32::Win32;

use flow_core::address::Address;
use flow_core::arch::Architecture;
use flow_core::mem::*;
use flow_core::ProcessTrait;

#[derive(Debug, Clone)]
pub struct Win32UserProcess {
    eprocess: Address,
    pid: i32,
    name: String,
    dtb: Address,
    wow64: Address,
    peb: Address,
    sys_arch: Architecture,
}

impl Win32UserProcess {
    pub fn try_with_eprocess<T>(
        mem: &mut T,
        win: &Win32,
        offsets: &Win32Offsets,
        eprocess: Address,
    ) -> Result<Self>
    where
        T: VirtualRead,
    {
        let mut reader = VirtualReader::with(mem, win.start_block.arch, win.start_block.dtb);

        let pid = reader.virt_read_i32(eprocess + offsets.eproc_pid)?;
        let name = reader.virt_read_cstr(eprocess + offsets.eproc_name, 16)?;
        let dtb = reader.virt_read_addr(eprocess + offsets.kproc_dtb)?;
        let wow64 = if offsets.eproc_wow64.is_zero() {
            Address::null()
        } else {
            reader.virt_read_addr(eprocess + offsets.eproc_wow64)?
        };

        // read peb

        let sys_arch = win.start_block.arch;

        Ok(Self {
            eprocess,
            pid,
            name,
            dtb,
            wow64,
            peb: Address::null(),
            sys_arch,
        })
    }

    pub fn try_with_name<T>(
        mem: &mut T,
        win: &Win32,
        offsets: &Win32Offsets,
        name: &str,
    ) -> Result<Self>
    where
        T: VirtualRead,
    {
        win.eprocess_list(mem, offsets)?
            .iter()
            .map(|eproc| Win32UserProcess::try_with_eprocess(mem, win, offsets, *eproc))
            .filter_map(Result::ok)
            .inspect(|p| println!("{:?} {:?}", p.pid(), p.name()))
            .filter(|p| p.name() == name)
            .nth(0)
            .ok_or_else(|| Error::new(format!("unable to find process {}", name)))
    }

    pub fn eprocess(&self) -> Address {
        self.eprocess
    }

    pub fn wow64(&self) -> Address {
        self.wow64
    }

    pub fn peb(&self) -> Address {
        self.peb
    }
}

impl ProcessTrait for Win32UserProcess {
    fn pid(&self) -> i32 {
        self.pid
    }

    fn name(&self) -> String {
        self.name.clone()
    }

    fn dtb(&self) -> Address {
        self.dtb
    }
}
