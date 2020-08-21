use std::prelude::v1::*;

use super::{make_virt_mem, Kernel, Win32ModuleInfo};
use crate::error::{Error, Result};
use crate::win32::VirtualReadUnicodeString;

use std::fmt;

use memflow_core::architecture::x86;
use memflow_core::architecture::{AddressTranslator, Architecture};
use memflow_core::mem::{PhysicalMemory, VirtualFromPhysical, VirtualMemory, VirtualTranslate};
use memflow_core::types::Address;
use memflow_core::{OsProcessInfo, OsProcessModuleInfo};
use std::ptr;

use log::trace;

#[derive(Debug, Clone)]
//#[cfg_attr(feature = "serde", derive(::serde::Serialize))]
pub struct Win32ProcessInfo {
    pub address: Address,

    // general information from eprocess
    pub pid: i32,
    pub name: String,
    pub dtb: Address,
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
}

impl OsProcessInfo for Win32ProcessInfo {
    fn address(&self) -> Address {
        self.address
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

    fn sys_arch(&self) -> &dyn Architecture {
        self.sys_arch
    }

    fn proc_arch(&self) -> &dyn Architecture {
        self.proc_arch
    }
}

//#[derive(Clone)]
pub struct Win32Process<'a> {
    pub virt_mem: Box<dyn VirtualMemory + 'a>,
    pub proc_info: Win32ProcessInfo,
}

impl<'a> Win32Process<'a> {
    pub fn with_kernel<T: PhysicalMemory + 'a, V: VirtualTranslate + 'a>(
        kernel: Kernel<T, V>,
        proc_info: Win32ProcessInfo,
    ) -> Self {
        Self {
            virt_mem: make_virt_mem::<'a, _, _>(
                kernel.phys_mem,
                kernel.vat,
                proc_info.proc_arch,
                proc_info.sys_arch,
                proc_info.dtb,
            ),
            proc_info,
        }
    }

    /// Constructs a new process by borrowing a kernel object.
    ///
    /// Internally this will create a `VirtualFromPhysical` object that also
    /// borrows the PhysicalMemory and Vat objects from the kernel.
    ///
    /// The resulting process object is NOT cloneable due to the mutable borrowing.
    ///
    /// When u need a cloneable Process u have to use the `::with_kernel` function
    /// which will move the kernel object.
    pub fn with_kernel_ref<T: PhysicalMemory + 'a, V: VirtualTranslate + 'a>(
        kernel: &'a mut Kernel<T, V>,
        proc_info: Win32ProcessInfo,
    ) -> Self {
        Self {
            virt_mem: make_virt_mem::<'a, _, _>(
                &mut kernel.phys_mem,
                &mut kernel.vat,
                proc_info.proc_arch,
                proc_info.sys_arch,
                proc_info.dtb,
            ),
            proc_info,
        }
    }

    pub fn peb_list(&mut self) -> Result<Vec<Address>> {
        let mut list = Vec::new();

        let list_start = self.proc_info.peb_module;
        let mut list_entry = list_start;
        loop {
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

    pub fn module_info_from_peb(&mut self, peb_module: Address) -> Result<Win32ModuleInfo> {
        let base = match self.proc_info.proc_arch.bits() {
            64 => self
                .virt_mem
                .virt_read_addr64(peb_module + self.proc_info.ldr_data_base_offs)?,
            32 => self
                .virt_mem
                .virt_read_addr32(peb_module + self.proc_info.ldr_data_base_offs)?,
            _ => return Err(Error::InvalidArchitecture),
        };
        trace!("base={:x}", base);

        let size = match self.proc_info.proc_arch.bits() {
            64 => self
                .virt_mem
                .virt_read_addr64(peb_module + self.proc_info.ldr_data_size_offs)?
                .as_usize(),
            32 => self
                .virt_mem
                .virt_read_addr32(peb_module + self.proc_info.ldr_data_size_offs)?
                .as_usize(),
            _ => return Err(Error::InvalidArchitecture),
        };
        trace!("size={:x}", size);

        let name = self.virt_mem.virt_read_unicode_string(
            self.proc_info.proc_arch,
            peb_module + self.proc_info.ldr_data_name_offs,
        )?;
        trace!("name={}", name);

        Ok(Win32ModuleInfo {
            peb_module,
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

impl<'a> fmt::Debug for Win32Process<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self.proc_info)
    }
}
