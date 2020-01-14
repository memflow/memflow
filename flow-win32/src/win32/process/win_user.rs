// TODO: this module will be renamed / replaces user.rs when its finished
// this will construct a user process by itself and doesnt hold any additional data

use crate::error::{Error, Result};
use log::trace;

use crate::offsets::Win32Offsets;
use crate::win32::{process::Win32Process, Win32};

use flow_core::address::Address;
use flow_core::arch::{Architecture, InstructionSet};
use flow_core::mem::*;
use flow_core::ProcessTrait;

// TODO: put this in process/user.rs
#[derive(Debug, Clone)]
pub struct Win32UserProcess {
    eprocess: Address,
    pid: i32,
    name: String,
    dtb: Address,
    wow64: Address,
    peb: Address,
    peb_module: Address,
    sys_arch: Architecture,
    proc_arch: Architecture,
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
        trace!("pid={}", pid);
        let name = reader.virt_read_cstr(eprocess + offsets.eproc_name, 16)?;
        trace!("name={}", name);
        let dtb = reader.virt_read_addr(eprocess + offsets.kproc_dtb)?;
        trace!("dtb={:x}", dtb);
        let wow64 = if offsets.eproc_wow64.is_zero() {
            Address::null()
        } else {
            reader.virt_read_addr(eprocess + offsets.eproc_wow64)?
        };
        trace!("wow64={:x}", wow64);

        // read peb
        let peb = if wow64.is_null() {
            trace!("reading peb for x64 process");
            reader.virt_read_addr(eprocess + offsets.eproc_peb)?
        } else {
            trace!("reading peb for wow64 process");
            reader.virt_read_addr(wow64)?
        };
        trace!("peb={:x}", peb);

        let sys_arch = win.start_block.arch;
        trace!("sys_arch={:?}", sys_arch);
        let proc_arch = Architecture::from(match sys_arch.instruction_set {
            InstructionSet::X64 => {
                if wow64.is_null() {
                    InstructionSet::X64
                } else {
                    InstructionSet::X86
                }
            }
            InstructionSet::X86Pae => InstructionSet::X86,
            InstructionSet::X86 => InstructionSet::X86,
            _ => return Err(Error::new("invalid architecture")),
        });
        trace!("proc_arch={:?}", proc_arch);

        // from here on out we are in the process context
        // we will be using the process type architecture now
        let (peb_ldr_offs, ldr_list_offs) = match proc_arch.instruction_set {
            InstructionSet::X64 => (offsets.peb_ldr_x64, offsets.ldr_list_x64),
            InstructionSet::X86 => (offsets.peb_ldr_x86, offsets.ldr_list_x86),
            _ => return Err(Error::new("invalid architecture")),
        };
        trace!("peb_ldr_offs={:x}", peb_ldr_offs);
        trace!("ldr_list_offs={:x}", ldr_list_offs);

        // construct reader with process dtb
        let mut proc_reader = VirtualReader::with(mem, win.start_block.arch, dtb);
        let peb_ldr = match proc_arch.instruction_set {
            InstructionSet::X64 => proc_reader.virt_read_addr64(peb + peb_ldr_offs)?,
            InstructionSet::X86 => proc_reader.virt_read_addr32(peb + peb_ldr_offs)?,
            _ => return Err(Error::new("invalid architecture")),
        };
        trace!("peb_ldr={:x}", peb_ldr);

        let peb_module = match proc_arch.instruction_set {
            InstructionSet::X64 => proc_reader.virt_read_addr64(peb_ldr + ldr_list_offs)?,
            InstructionSet::X86 => proc_reader.virt_read_addr32(peb_ldr + ldr_list_offs)?,
            _ => return Err(Error::new("invalid architecture")),
        };
        trace!("peb_module={:x}", peb_module);

        Ok(Self {
            eprocess,
            pid,
            name,
            dtb,
            wow64,
            peb,
            peb_module,
            sys_arch,
            proc_arch,
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
            .inspect(|p| trace!("{} {}", p.pid(), p.name()))
            .filter(|p| p.name() == name)
            .nth(0)
            .ok_or_else(|| Error::new(format!("unable to find process {}", name)))
    }
}

impl Win32Process for Win32UserProcess {
    fn wow64(&self) -> Address {
        self.wow64
    }

    fn peb(&self) -> Address {
        self.peb
    }

    fn peb_module(&self) -> Address {
        self.peb_module
    }

    fn peb_list<T: VirtualRead>(
        &self,
        mem: &mut T,
        offsets: &Win32Offsets,
    ) -> Result<Vec<Address>> {
        let mut proc_reader = VirtualReader::with(mem, self.sys_arch, self.dtb);

        let mut pebs = Vec::new();

        println!("self {:?}", self);

        let mut peb_module = self.peb_module;
        loop {
            let next = proc_reader.virt_read_addr(peb_module + offsets.list_blink)?;
            if next.is_null() || next == self.peb_module {
                break;
            }
            pebs.push(next);
            peb_module = next;
        }

        Ok(pebs)
    }
}

impl ProcessTrait for Win32UserProcess {
    fn address(&self) -> Address {
        self.eprocess
    }

    fn pid(&self) -> i32 {
        self.pid
    }

    fn name(&self) -> String {
        self.name.clone()
    }

    fn dtb(&self) -> Address {
        self.dtb
    }

    fn sys_arch(&self) -> Architecture {
        self.sys_arch
    }

    fn proc_arch(&self) -> Architecture {
        self.proc_arch
    }
}
