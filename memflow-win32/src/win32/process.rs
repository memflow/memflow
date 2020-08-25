use std::prelude::v1::*;

use super::{Kernel, Win32ModuleInfo};
use crate::error::{Error, Result};
use crate::win32::VirtualReadUnicodeString;

use log::trace;
use std::fmt;

use memflow_core::architecture::Architecture;
use memflow_core::mem::{PhysicalMemory, VirtualDMA, VirtualMemory, VirtualTranslate};
use memflow_core::process::{OsProcessInfo, OsProcessModuleInfo, PID};
use memflow_core::types::Address;

use super::Win32VirtualTranslate;

/// Exit status of a win32 process
pub type Win32ExitStatus = i32;

/// Process has not exited yet
pub const EXIT_STATUS_STILL_ACTIVE: i32 = 259;

const MAX_ITER_COUNT: usize = 65536;

#[derive(Debug, Clone)]
//#[cfg_attr(feature = "serde", derive(::serde::Serialize))]
pub struct Win32ProcessInfo {
    pub address: Address,

    // general information from eprocess
    pub pid: PID,
    pub name: String,
    pub dtb: Address,
    pub section_base: Address,
    pub exit_status: Win32ExitStatus,
    pub ethread: Address,
    pub wow64: Address,

    // teb
    pub teb: Address,

    // peb
    pub peb: Address,
    pub peb_module: Address,

    // architecture
    pub sys_arch: &'static dyn Architecture,
    pub proc_arch: &'static dyn Architecture,

    // offsets for this process (either x86 or x64 offsets)
    pub ldr_data_base_offs: usize,
    pub ldr_data_size_offs: usize,
    pub ldr_data_name_offs: usize,
}

impl Win32ProcessInfo {
    pub fn wow64(&self) -> Address {
        self.wow64
    }

    pub fn peb(&self) -> Address {
        self.peb
    }

    pub fn peb_module(&self) -> Address {
        self.peb_module
    }

    pub fn translator(&self) -> Win32VirtualTranslate {
        Win32VirtualTranslate::new(self.sys_arch, self.dtb)
    }
}

impl OsProcessInfo for Win32ProcessInfo {
    fn address(&self) -> Address {
        self.address
    }

    fn pid(&self) -> PID {
        self.pid
    }

    fn name(&self) -> String {
        self.name.clone()
    }

    fn sys_arch(&self) -> &'static dyn Architecture {
        self.sys_arch
    }

    fn proc_arch(&self) -> &'static dyn Architecture {
        self.proc_arch
    }
}

pub struct Win32Process<T> {
    pub virt_mem: T,
    pub proc_info: Win32ProcessInfo,
}

// TODO: can be removed i think
impl<T: Clone> Clone for Win32Process<T> {
    fn clone(&self) -> Self {
        Self {
            virt_mem: self.virt_mem.clone(),
            proc_info: self.proc_info.clone(),
        }
    }
}

// TODO: replace the following impls with a dedicated builder
// TODO: add non cloneable thing
impl<'a, T: PhysicalMemory, V: VirtualTranslate>
    Win32Process<VirtualDMA<T, V, Win32VirtualTranslate>>
{
    pub fn with_kernel(kernel: Kernel<T, V>, proc_info: Win32ProcessInfo) -> Self {
        let virt_mem = VirtualDMA::with_vat(
            kernel.phys_mem,
            proc_info.proc_arch,
            proc_info.translator(),
            kernel.vat,
        );

        Self {
            virt_mem,
            proc_info,
        }
    }

    /// Consume the self object and returns the containing memory connection
    pub fn destroy(self) -> T {
        self.virt_mem.destroy()
    }
}

impl<'a, T: PhysicalMemory, V: VirtualTranslate>
    Win32Process<VirtualDMA<&'a mut T, &'a mut V, Win32VirtualTranslate>>
{
    /// Constructs a new process by borrowing a kernel object.
    ///
    /// Internally this will create a `VirtualDMA` object that also
    /// borrows the PhysicalMemory and Vat objects from the kernel.
    ///
    /// The resulting process object is NOT cloneable due to the mutable borrowing.
    ///
    /// When u need a cloneable Process u have to use the `::with_kernel` function
    /// which will move the kernel object.
    pub fn with_kernel_ref(kernel: &'a mut Kernel<T, V>, proc_info: Win32ProcessInfo) -> Self {
        let virt_mem = VirtualDMA::with_vat(
            &mut kernel.phys_mem,
            proc_info.proc_arch,
            proc_info.translator(),
            &mut kernel.vat,
        );

        Self {
            virt_mem,
            proc_info,
        }
    }
}

impl<T: VirtualMemory> Win32Process<T> {
    pub fn peb_list(&mut self) -> Result<Vec<Address>> {
        let mut list = Vec::new();

        let list_start = self.proc_info.peb_module;
        let mut list_entry = list_start;
        for _ in 0..MAX_ITER_COUNT {
            list.push(list_entry);
            list_entry = match self.proc_info.proc_arch.bits() {
                64 => self.virt_mem.virt_read_addr64(list_entry)?,
                32 => self.virt_mem.virt_read_addr32(list_entry)?,
                _ => return Err(Error::InvalidArchitecture),
            };
            if list_entry.is_null() || list_entry == self.proc_info.peb_module {
                break;
            }
        }

        Ok(list)
    }

    pub fn module_info_from_peb(&mut self, peb_entry: Address) -> Result<Win32ModuleInfo> {
        let base = match self.proc_info.proc_arch.bits() {
            64 => self
                .virt_mem
                .virt_read_addr64(peb_entry + self.proc_info.ldr_data_base_offs)?,
            32 => self
                .virt_mem
                .virt_read_addr32(peb_entry + self.proc_info.ldr_data_base_offs)?,
            _ => return Err(Error::InvalidArchitecture),
        };
        trace!("base={:x}", base);

        let size = match self.proc_info.proc_arch.bits() {
            64 => self
                .virt_mem
                .virt_read_addr64(peb_entry + self.proc_info.ldr_data_size_offs)?
                .as_usize(),
            32 => self
                .virt_mem
                .virt_read_addr32(peb_entry + self.proc_info.ldr_data_size_offs)?
                .as_usize(),
            _ => return Err(Error::InvalidArchitecture),
        };
        trace!("size={:x}", size);

        let name = self.virt_mem.virt_read_unicode_string(
            self.proc_info.proc_arch,
            peb_entry + self.proc_info.ldr_data_name_offs,
        )?;
        trace!("name={}", name);

        Ok(Win32ModuleInfo {
            peb_entry,
            parent_eprocess: self.proc_info.address,
            base,
            size,
            name,
        })
    }

    pub fn module_info_list(&mut self) -> Result<Vec<Win32ModuleInfo>> {
        let mut list = Vec::new();
        for &peb in self.peb_list()?.iter() {
            if let Ok(modu) = self.module_info_from_peb(peb) {
                list.push(modu);
            }
        }
        Ok(list)
    }

    pub fn module_info(&mut self, name: &str) -> Result<Win32ModuleInfo> {
        let module_info_list = self.module_info_list()?;
        module_info_list
            .into_iter()
            .inspect(|module| trace!("{:x} {}", module.base(), module.name()))
            .find(|module| module.name() == name)
            .ok_or_else(|| Error::ModuleInfo)
    }
}

impl<T> fmt::Debug for Win32Process<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self.proc_info)
    }
}
