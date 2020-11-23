use std::prelude::v1::*;

use super::{Kernel, Win32ModuleInfo};
use crate::error::{Error, Result};
use crate::offsets::Win32ArchOffsets;
use crate::win32::VirtualReadUnicodeString;

use log::trace;
use std::fmt;

use memflow::architecture::ArchitectureObj;
use memflow::mem::{PhysicalMemory, VirtualDMA, VirtualMemory, VirtualTranslate};
use memflow::process::{OsProcessInfo, OsProcessModuleInfo, PID};
use memflow::types::Address;

use super::Win32VirtualTranslate;

/// Exit status of a win32 process
pub type Win32ExitStatus = i32;

/// Process has not exited yet
pub const EXIT_STATUS_STILL_ACTIVE: i32 = 259;

/// EPROCESS ImageFileName byte length
pub const IMAGE_FILE_NAME_LENGTH: usize = 15;

const MAX_ITER_COUNT: usize = 65536;

#[derive(Debug, Clone, Copy)]
#[repr(C)]
#[cfg_attr(feature = "serde", derive(::serde::Serialize))]
pub struct Win32ModuleListInfo {
    module_base: Address,
    offsets: Win32ArchOffsets,
}

impl Win32ModuleListInfo {
    pub fn with_peb<V: VirtualMemory>(
        mem: &mut V,
        peb: Address,
        arch: ArchitectureObj,
    ) -> Result<Win32ModuleListInfo> {
        let offsets = Win32ArchOffsets::from(arch);

        trace!("peb_ldr_offs={:x}", offsets.peb_ldr);
        trace!("ldr_list_offs={:x}", offsets.ldr_list);

        let peb_ldr = mem.virt_read_addr_arch(arch, peb + offsets.peb_ldr)?;
        trace!("peb_ldr={:x}", peb_ldr);

        let module_base = mem.virt_read_addr_arch(arch, peb_ldr + offsets.ldr_list)?;

        Self::with_base(module_base, arch)
    }

    pub fn with_base(module_base: Address, arch: ArchitectureObj) -> Result<Win32ModuleListInfo> {
        trace!("module_base={:x}", module_base);

        let offsets = Win32ArchOffsets::from(arch);
        trace!("offsets={:?}", offsets);

        Ok(Win32ModuleListInfo {
            module_base,
            offsets,
        })
    }

    pub fn module_base(&self) -> Address {
        self.module_base
    }

    pub fn module_entry_list<V: VirtualMemory>(
        &self,
        mem: &mut V,
        arch: ArchitectureObj,
    ) -> Result<Vec<Address>> {
        let mut list = Vec::new();

        let list_start = self.module_base;
        let mut list_entry = list_start;
        for _ in 0..MAX_ITER_COUNT {
            list.push(list_entry);
            list_entry = mem.virt_read_addr_arch(arch, list_entry)?;
            // Break on misaligned entry. On NT 4.0 list end is misaligned, maybe it's a flag?
            if list_entry.is_null()
                || (list_entry.as_u64() & 0b111) != 0
                || list_entry == self.module_base
            {
                break;
            }
        }

        Ok(list)
    }

    pub fn module_info_from_entry<V: VirtualMemory>(
        &self,
        entry: Address,
        parent_eprocess: Address,
        mem: &mut V,
        arch: ArchitectureObj,
    ) -> Result<Win32ModuleInfo> {
        let base = mem.virt_read_addr_arch(arch, entry + self.offsets.ldr_data_base)?;

        trace!("base={:x}", base);

        let size = mem
            .virt_read_addr_arch(arch, entry + self.offsets.ldr_data_size)?
            .as_usize();

        trace!("size={:x}", size);

        let path = mem.virt_read_unicode_string(arch, entry + self.offsets.ldr_data_full_name)?;
        trace!("path={}", path);

        let name = mem.virt_read_unicode_string(arch, entry + self.offsets.ldr_data_base_name)?;
        trace!("name={}", name);

        Ok(Win32ModuleInfo {
            peb_entry: entry,
            parent_eprocess,
            base,
            size,
            path,
            name,
        })
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(::serde::Serialize))]
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
    pub teb: Option<Address>,
    pub teb_wow64: Option<Address>,

    // peb
    pub peb_native: Address,
    pub peb_wow64: Option<Address>,

    // modules
    pub module_info_native: Win32ModuleListInfo,
    pub module_info_wow64: Option<Win32ModuleListInfo>,

    // architecture
    pub sys_arch: ArchitectureObj,
    pub proc_arch: ArchitectureObj,
}

impl Win32ProcessInfo {
    pub fn wow64(&self) -> Address {
        self.wow64
    }

    pub fn peb(&self) -> Address {
        if let Some(peb) = self.peb_wow64 {
            peb
        } else {
            self.peb_native
        }
    }

    pub fn peb_native(&self) -> Address {
        self.peb_native
    }

    pub fn peb_wow64(&self) -> Option<Address> {
        self.peb_wow64
    }

    /// Return the module list information of process native architecture
    ///
    /// If the process is a wow64 process, module_info_wow64 is returned, otherwise, module_info_native is
    /// returned.
    pub fn module_info(&self) -> Win32ModuleListInfo {
        if !self.wow64.is_null() {
            self.module_info_wow64.unwrap()
        } else {
            self.module_info_native
        }
    }

    pub fn module_info_native(&self) -> Win32ModuleListInfo {
        self.module_info_native
    }

    pub fn module_info_wow64(&self) -> Option<Win32ModuleListInfo> {
        self.module_info_wow64
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

    fn sys_arch(&self) -> ArchitectureObj {
        self.sys_arch
    }

    fn proc_arch(&self) -> ArchitectureObj {
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
    fn module_list_with_infos_extend<
        E: Extend<Win32ModuleInfo>,
        I: Iterator<Item = (Win32ModuleListInfo, ArchitectureObj)>,
    >(
        &mut self,
        module_infos: I,
        out: &mut E,
    ) -> Result<()> {
        for (info, arch) in module_infos {
            out.extend(
                info.module_entry_list(&mut self.virt_mem, arch)?
                    .iter()
                    .filter_map(|&peb| {
                        info.module_info_from_entry(
                            peb,
                            self.proc_info.address,
                            &mut self.virt_mem,
                            arch,
                        )
                        .ok()
                    }),
            );
        }
        Ok(())
    }

    pub fn module_entry_list(&mut self) -> Result<Vec<Address>> {
        let (info, arch) = if let Some(info_wow64) = self.proc_info.module_info_wow64 {
            (info_wow64, self.proc_info.proc_arch)
        } else {
            (self.proc_info.module_info_native, self.proc_info.sys_arch)
        };

        info.module_entry_list(&mut self.virt_mem, arch)
    }

    pub fn module_entry_list_native(&mut self) -> Result<Vec<Address>> {
        let (info, arch) = (self.proc_info.module_info_native, self.proc_info.sys_arch);
        info.module_entry_list(&mut self.virt_mem, arch)
    }

    pub fn module_entry_list_wow64(&mut self) -> Result<Vec<Address>> {
        let (info, arch) = (
            self.proc_info
                .module_info_wow64
                .ok_or(Error::Other("WoW64 module list does not exist"))?,
            self.proc_info.proc_arch,
        );
        info.module_entry_list(&mut self.virt_mem, arch)
    }

    pub fn module_list(&mut self) -> Result<Vec<Win32ModuleInfo>> {
        let mut vec = Vec::new();
        self.module_list_extend(&mut vec)?;
        Ok(vec)
    }

    pub fn module_list_extend<E: Extend<Win32ModuleInfo>>(&mut self, out: &mut E) -> Result<()> {
        let infos = [
            (
                Some(self.proc_info.module_info_native),
                self.proc_info.sys_arch,
            ),
            (self.proc_info.module_info_wow64, self.proc_info.proc_arch),
        ];

        let iter = infos
            .iter()
            .cloned()
            .filter_map(|(info, arch)| info.map(|info| (info, arch)));

        self.module_list_with_infos_extend(iter, out)
    }

    pub fn main_module_info(&mut self) -> Result<Win32ModuleInfo> {
        let module_list = self.module_list()?;
        module_list
            .into_iter()
            .inspect(|module| trace!("{:x} {}", module.base(), module.name()))
            .find(|module| module.base == self.proc_info.section_base)
            .ok_or_else(|| Error::ModuleInfo)
    }

    pub fn module_info(&mut self, name: &str) -> Result<Win32ModuleInfo> {
        let module_list = self.module_list()?;
        module_list
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
