use crate::error::{Error, Result};

use crate::offsets::Win32Offsets;
use crate::win32::{process::Win32Process, Win32};

use flow_core::address::{Address, Length};
use flow_core::arch::Architecture;
use flow_core::mem::*;
use flow_core::ProcessTrait;

use pelite::{self, pe64::exports::Export, PeView};

#[derive(Debug, Clone)]
pub struct Win32KernelProcess {
    base: Address,
    dtb: Address,
    peb_module: Address,
    sys_arch: Architecture,
}

impl Win32KernelProcess {
    pub fn try_with<T>(mem: &mut T, win: &Win32) -> Result<Self>
    where
        T: AccessVirtualMemory,
    {
        let mut reader = VirtualMemoryContext::with(mem, win.start_block.arch, win.start_block.dtb);

        // TODO: move this to Win32::try_with() at one point

        // read pe header
        let mut pe_buf = vec![0; win.kernel_size.as_usize()];
        reader.virt_read_raw_into(win.kernel_base, &mut pe_buf)?;

        let pe = PeView::from_bytes(&pe_buf)?;

        // find PsActiveProcessHead
        let loaded_module_list = match pe.get_export_by_name("PsLoadedModuleList")? {
            Export::Symbol(s) => win.kernel_base + Length::from(*s),
            Export::Forward(_) => {
                return Err(Error::new(
                    "PsLoadedModuleList found but it was a forwarded export",
                ))
            }
        };

        let peb_module = reader.virt_read_addr(loaded_module_list)?;

        Ok(Self {
            base: win.kernel_base,
            dtb: win.start_block.dtb,
            peb_module,
            sys_arch: win.start_block.arch,
        })
    }
}

impl Win32Process for Win32KernelProcess {
    // TODO: does wow64 and peb really need to be in Win32Process
    fn wow64(&self) -> Address {
        Address::null()
    }

    // TODO: does peb really need to be exposed?
    fn peb(&self) -> Address {
        Address::null()
    }

    fn peb_module(&self) -> Address {
        self.peb_module
    }

    fn peb_list<T: AccessVirtualMemory>(&self, mem: &mut T) -> Result<Vec<Address>> {
        let mut proc_reader = VirtualMemoryContext::with(mem, self.sys_arch, self.dtb);

        let mut pebs = Vec::new();

        let list_start = self.peb_module;
        let mut list_entry = list_start;
        loop {
            pebs.push(list_entry);
            list_entry = proc_reader.virt_read_addr(list_entry)?;
            if list_entry.is_null() || list_entry == self.peb_module {
                break;
            }
        }

        Ok(pebs)
    }
}

impl ProcessTrait for Win32KernelProcess {
    fn address(&self) -> Address {
        self.base
    }

    fn pid(&self) -> i32 {
        0
    }

    fn name(&self) -> String {
        "ntoskrnl.exe".to_owned()
    }

    fn dtb(&self) -> Address {
        self.dtb
    }

    fn sys_arch(&self) -> Architecture {
        self.sys_arch
    }

    fn proc_arch(&self) -> Architecture {
        self.sys_arch
    }
}
