use std::prelude::v1::*;

use super::{KernelInfo, Win32Process, Win32ProcessInfo};
use crate::error::{Error, Result};
use crate::offsets::Win32Offsets;
use crate::pe::{pe32, pe64, MemoryPeViewContext};

use log::trace;
use std::fmt;

use flow_core::architecture::Architecture;
use flow_core::mem::{PhysicalMemory, VirtualFromPhysical, VirtualMemory, VirtualTranslate};
use flow_core::process::{OperatingSystem, OsProcessInfo, OsProcessModuleInfo};
use flow_core::types::Address;

use pelite::{
    self,
    pe32::exports::GetProcAddress as GetProcAddress32,
    pe64::exports::{Export, GetProcAddress},
};

#[derive(Clone)]
pub struct Kernel<T: PhysicalMemory, V: VirtualTranslate> {
    pub phys_mem: T,
    pub vat: V,
    pub offsets: Win32Offsets,

    pub kernel_info: KernelInfo,
}

impl<T: PhysicalMemory, V: VirtualTranslate> OperatingSystem for Kernel<T, V> {}

impl<T: PhysicalMemory, V: VirtualTranslate> Kernel<T, V> {
    pub fn new(phys_mem: T, vat: V, offsets: Win32Offsets, kernel_info: KernelInfo) -> Self {
        Self {
            phys_mem,
            vat,
            offsets,

            kernel_info,
        }
    }

    /// Consume the self object and return the containing memory connection
    pub fn destroy(self) -> T {
        self.phys_mem
    }

    pub fn eprocess_list(&mut self) -> Result<Vec<Address>> {
        // TODO: create a VirtualFromPhysical constructor for kernel_info
        let mut reader = VirtualFromPhysical::with_vat(
            &mut self.phys_mem,
            self.kernel_info.start_block.arch,
            self.kernel_info.start_block.arch,
            self.kernel_info.kernel_dtb,
            &mut self.vat,
        );

        let mut eprocs = Vec::new();

        let list_start = self.kernel_info.eprocess_base + self.offsets.eproc_link;
        let mut list_entry = list_start;

        loop {
            let eprocess = list_entry - self.offsets.eproc_link;
            trace!("eprocess={}", eprocess);

            // test flink + blink before adding the process
            let flink_entry = reader.virt_read_addr(list_entry)?;
            trace!("flink_entry={}", flink_entry);
            let blink_entry = reader.virt_read_addr(list_entry + self.offsets.list_blink)?;
            trace!("blink_entry={}", blink_entry);

            if flink_entry.is_null() || blink_entry.is_null() || flink_entry == list_start {
                break;
            }

            trace!("found eprocess {:x}", eprocess);
            eprocs.push(eprocess);

            // continue
            list_entry = flink_entry;
        }

        trace!("found {} eprocesses", eprocs.len());
        Ok(eprocs)
    }

    pub fn ntoskrnl_process_info(&mut self) -> Result<Win32ProcessInfo> {
        // TODO: create a VirtualFromPhysical constructor for kernel_info
        let mut reader = VirtualFromPhysical::with_vat(
            &mut self.phys_mem,
            self.kernel_info.start_block.arch,
            self.kernel_info.start_block.arch,
            self.kernel_info.kernel_dtb,
            &mut self.vat,
        );

        // TODO: cache pe globally
        // find PsLoadedModuleList
        let loaded_module_list = {
            // TODO: use pe wrap :)
            let pectx = MemoryPeViewContext::new(&mut reader, self.kernel_info.kernel_base)
                .map_err(Error::from)?;
            match self.kernel_info.start_block.arch.bits() {
                32 => {
                    let pe = pe32::MemoryPeView::new(&pectx).map_err(Error::from)?;
                    match pe.get_export("PsLoadedModuleList").map_err(Error::from)? {
                        Export::Symbol(s) => self.kernel_info.kernel_base + *s as usize,
                        Export::Forward(_) => {
                            return Err(Error::Other(
                                "PsLoadedModuleList found but it was a forwarded export",
                            ))
                        }
                    }
                }
                64 => {
                    let pe = pe64::MemoryPeView::new(&pectx).map_err(Error::from)?;
                    match pe.get_export("PsLoadedModuleList").map_err(Error::from)? {
                        Export::Symbol(s) => self.kernel_info.kernel_base + *s as usize,
                        Export::Forward(_) => {
                            return Err(Error::Other(
                                "PsLoadedModuleList found but it was a forwarded export",
                            ))
                        }
                    }
                }
                _ => return Err(Error::InvalidArchitecture),
            }
        };

        let peb_module = reader.virt_read_addr(loaded_module_list)?;

        // determine the offsets to be used when working with this process
        let (ldr_data_base_offs, ldr_data_size_offs, ldr_data_name_offs) =
            match self.kernel_info.start_block.arch.bits() {
                64 => (
                    self.offsets.ldr_data_base_x64,
                    self.offsets.ldr_data_size_x64,
                    self.offsets.ldr_data_name_x64,
                ),
                32 => (
                    self.offsets.ldr_data_base_x86,
                    self.offsets.ldr_data_size_x86,
                    self.offsets.ldr_data_name_x86,
                ),
                _ => return Err(Error::InvalidArchitecture),
            };
        trace!("ldr_data_base_offs={:x}", ldr_data_base_offs);
        trace!("ldr_data_size_offs={:x}", ldr_data_size_offs);
        trace!("ldr_data_name_offs={:x}", ldr_data_name_offs);

        Ok(Win32ProcessInfo {
            address: self.kernel_info.kernel_base,

            pid: 0,
            name: "ntoskrnl.exe".to_string(),
            dtb: self.kernel_info.kernel_dtb,
            ethread: Address::NULL, // TODO: see below
            wow64: Address::NULL,

            teb: Address::NULL, // TODO: see below

            peb: Address::NULL,
            peb_module,

            sys_arch: self.kernel_info.start_block.arch,
            proc_arch: self.kernel_info.start_block.arch,

            ldr_data_base_offs,
            ldr_data_size_offs,
            ldr_data_name_offs,
        })
    }

    pub fn process_info_from_eprocess(&mut self, eprocess: Address) -> Result<Win32ProcessInfo> {
        // TODO: create a VirtualFromPhysical constructor for kernel_info
        let mut reader = VirtualFromPhysical::with_vat(
            &mut self.phys_mem,
            self.kernel_info.start_block.arch,
            self.kernel_info.start_block.arch,
            self.kernel_info.kernel_dtb,
            &mut self.vat,
        );

        let pid: i32 = reader.virt_read(eprocess + self.offsets.eproc_pid)?;
        trace!("pid={}", pid);

        let name = reader.virt_read_cstr(eprocess + self.offsets.eproc_name, 16)?;
        trace!("name={}", name);

        let dtb = reader.virt_read_addr(eprocess + self.offsets.kproc_dtb)?;
        trace!("dtb={:x}", dtb);

        let wow64 = if self.offsets.eproc_wow64 == 0 {
            trace!("eproc_wow64=null; skipping wow64 detection");
            Address::null()
        } else {
            trace!(
                "eproc_wow64={:x}; trying to read wow64 pointer",
                self.offsets.eproc_wow64
            );
            reader.virt_read_addr(eprocess + self.offsets.eproc_wow64)?
        };
        trace!("wow64={:x}", wow64);

        // determine process architecture
        let sys_arch = self.kernel_info.start_block.arch;
        trace!("sys_arch={:?}", sys_arch);
        let proc_arch = match sys_arch.bits() {
            64 => {
                if wow64.is_null() {
                    Architecture::X64
                } else {
                    Architecture::X86
                }
            }
            32 => Architecture::X86,
            _ => return Err(Error::InvalidArchitecture),
        };
        trace!("proc_arch={:?}", proc_arch);

        // read native_peb (either the process peb or the peb containing the wow64 helpers)
        let native_peb = reader.virt_read_addr(eprocess + self.offsets.eproc_peb)?;
        trace!("native_peb={:x}", native_peb);

        // find first ethread
        let ethread = reader.virt_read_addr(eprocess + self.offsets.eproc_thread_list)?
            - self.offsets.ethread_list_entry;
        trace!("ethread={:x}", ethread);

        // TODO: does this need to be read with the process ctx?
        let teb = if wow64.is_null() {
            reader.virt_read_addr(ethread + self.offsets.kthread_teb)?
        } else {
            reader.virt_read_addr(ethread + self.offsets.kthread_teb)? + 0x2000
        };
        trace!("teb={:x}", teb);

        // construct reader with process dtb
        // TODO: can tlb be used here already?
        let mut proc_reader = VirtualFromPhysical::new(
            &mut self.phys_mem,
            self.kernel_info.start_block.arch,
            proc_arch,
            dtb,
        );

        // from here on out we are in the process context
        // we will be using the process type architecture now
        let real_peb = if wow64.is_null() {
            proc_reader.virt_read_addr(teb + self.offsets.teb_peb)?
        } else {
            proc_reader.virt_read_addr(teb + self.offsets.teb_peb_x86)?
        };
        trace!("real_peb={:x}", real_peb);

        // retrieve peb offsets
        let (peb_ldr_offs, ldr_list_offs) = match proc_arch.bits() {
            64 => (self.offsets.peb_ldr_x64, self.offsets.ldr_list_x64),
            32 => (self.offsets.peb_ldr_x86, self.offsets.ldr_list_x86),
            _ => return Err(Error::InvalidArchitecture),
        };
        trace!("peb_ldr_offs={:x}", peb_ldr_offs);
        trace!("ldr_list_offs={:x}", ldr_list_offs);

        let peb_ldr =
            proc_reader.virt_read_addr(real_peb /* TODO: can we have both? */ + peb_ldr_offs)?;
        trace!("peb_ldr={:x}", peb_ldr);

        let peb_module = proc_reader.virt_read_addr(peb_ldr + ldr_list_offs)?;
        trace!("peb_module={:x}", peb_module);

        // determine the offsets to be used when working with this process
        let (ldr_data_base_offs, ldr_data_size_offs, ldr_data_name_offs) = match proc_arch.bits() {
            64 => (
                self.offsets.ldr_data_base_x64,
                self.offsets.ldr_data_size_x64,
                self.offsets.ldr_data_name_x64,
            ),
            32 => (
                self.offsets.ldr_data_base_x86,
                self.offsets.ldr_data_size_x86,
                self.offsets.ldr_data_name_x86,
            ),
            _ => return Err(Error::InvalidArchitecture),
        };
        trace!("ldr_data_base_offs={:x}", ldr_data_base_offs);
        trace!("ldr_data_size_offs={:x}", ldr_data_size_offs);
        trace!("ldr_data_name_offs={:x}", ldr_data_name_offs);

        Ok(Win32ProcessInfo {
            address: eprocess,

            pid,
            name,
            dtb,
            ethread,
            wow64,

            teb,

            peb: real_peb, // TODO: store native + real peb
            peb_module,

            sys_arch,
            proc_arch,

            ldr_data_base_offs,
            ldr_data_size_offs,
            ldr_data_name_offs,
        })
    }

    pub fn process_info_list(&mut self) -> Result<Vec<Win32ProcessInfo>> {
        let mut list = Vec::new();
        for &eprocess in self.eprocess_list()?.iter() {
            if let Ok(proc) = self.process_info_from_eprocess(eprocess) {
                list.push(proc);
            }
        }
        Ok(list)
    }

    pub fn process_info(&mut self, name: &str) -> Result<Win32ProcessInfo> {
        let process_info_list = self.process_info_list()?;
        let candidates = process_info_list
            .iter()
            .inspect(|process| trace!("{} {}", process.pid(), process.name()))
            .filter(|process| {
                process.name().to_lowercase() == name[..name.len().min(14)].to_lowercase()
            })
            .collect::<Vec<_>>();

        for &candidate in candidates.iter() {
            // TODO: properly probe pe header here and check ImageBase
            // TODO: this wont work with tlb
            trace!("inspecting candidate process: {:?}", candidate);
            let mut process = Win32Process::with_kernel(self, candidate.clone());
            if process
                .module_info_list()?
                .iter()
                .inspect(|&module| trace!("{:x} {}", module.base(), module.name()))
                .find(|&module| module.name().to_lowercase() == name.to_lowercase())
                .ok_or_else(|| Error::ModuleInfo)
                .is_ok()
            {
                return Ok(candidate.clone());
            }
        }

        Err(Error::ProcessInfo)
    }
}

impl<T: PhysicalMemory, V: VirtualTranslate> fmt::Debug for Kernel<T, V> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self.kernel_info)
    }
}
