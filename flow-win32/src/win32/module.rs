/*
pub mod iter;
pub use iter::*;

pub mod export;
pub use export::*;

pub mod section;
pub use section::*;
*/
use crate::error::{Error, Result};
use log::trace;

use std::cell::RefCell;
use std::rc::Rc;

use flow_core::address::{Address, Length};
use flow_core::arch::{Architecture, InstructionSet};
use flow_core::mem::*;
use flow_core::process::ModuleTrait;
use flow_core::*;

use crate::offsets::Win32Offsets;
use crate::win32::*;

use pelite::{self, pe64::exports, PeView};

#[derive(Debug, Clone)]
pub struct Win32Module {
    peb_module: Address,
    parent_eprocess: Address, // parent "reference"

    base: Address,
    size: Length,
    name: String,
    // exports
    // sections
}

impl Win32Module {
    pub fn try_with_peb<T, U>(
        mem: &mut T,
        process: &U,
        offsets: &Win32Offsets,
        peb_module: Address,
    ) -> Result<Self>
    where
        T: VirtualMemoryTrait,
        U: ProcessTrait,
    {
        let mut proc_reader = VirtualMemory::with_proc_arch(
            mem,
            process.sys_arch(),
            process.proc_arch(),
            process.dtb(),
        );

        let base = match process.proc_arch().instruction_set {
            InstructionSet::X64 => {
                proc_reader.virt_read_addr64(peb_module + offsets.ldr_data_base_x64)?
            }
            InstructionSet::X86 => {
                proc_reader.virt_read_addr32(peb_module + offsets.ldr_data_base_x86)?
            }
            _ => return Err(Error::new("invalid architecture")),
        };
        trace!("base={:x}", base);

        let size = match process.proc_arch().instruction_set {
            InstructionSet::X64 => {
                let mut s = 0u64;
                proc_reader.virt_read_pod(peb_module + offsets.ldr_data_size_x64, &mut s)?;
                Length::from(s)
            }
            InstructionSet::X86 => {
                let mut s = 0u32;
                proc_reader.virt_read_pod(peb_module + offsets.ldr_data_size_x86, &mut s)?;
                Length::from(s)
            }
            _ => return Err(Error::new("invalid architecture")),
        };
        trace!("size={:x}", size);

        let name = match process.proc_arch().instruction_set {
            InstructionSet::X64 => {
                proc_reader.virt_read_unicode_string(peb_module + offsets.ldr_data_name_x64)?
            }
            InstructionSet::X86 => {
                proc_reader.virt_read_unicode_string(peb_module + offsets.ldr_data_name_x86)?
            }
            _ => return Err(Error::new("invalid architecture")),
        };
        trace!("name={}", name);

        Ok(Self {
            peb_module,
            parent_eprocess: process.address(),
            base,
            size,
            name,
        })
    }

    pub fn try_with_name<T, U>(
        mem: &mut T,
        process: &U,
        offsets: &Win32Offsets,
        name: &str,
    ) -> Result<Self>
    where
        T: VirtualMemoryTrait,
        U: ProcessTrait + Win32Process,
    {
        process
            .peb_list(mem, offsets)?
            .iter()
            .map(|peb| Win32Module::try_with_peb(mem, process, offsets, *peb))
            .filter_map(Result::ok)
            .inspect(|p| trace!("{:x} {}", p.base(), p.name()))
            .filter(|p| p.name() == name)
            .nth(0)
            .ok_or_else(|| Error::new(format!("unable to find process {}", name)))
    }
}

impl ModuleTrait for Win32Module {
    fn address(&self) -> Address {
        self.peb_module
    }

    fn parent_process(&self) -> Address {
        self.parent_eprocess
    }

    fn base(&self) -> Address {
        self.base
    }

    fn size(&self) -> Length {
        self.size
    }

    fn name(&self) -> String {
        self.name.clone()
    }
}
