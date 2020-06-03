use super::{KernelInfo, Win32Process, Win32ProcessInfo};
use crate::error::{Error, Result};
use crate::offsets::Win32Offsets;

use log::trace;
use std::fmt;

use flow_core::architecture::Architecture;
use flow_core::mem::{PhysicalMemory, VirtualFromPhysical, VirtualMemory, VAT};
use flow_core::process::{OperatingSystem, OsProcessInfo, OsProcessModuleInfo};
use flow_core::types::{Address, Length};

use pelite::{self, pe64::exports::Export, PeView};

#[derive(Clone)]
pub struct Kernel<T: PhysicalMemory, V: VAT> {
    pub phys_mem: T,
    pub vat: V,
    pub offsets: Win32Offsets,

    pub kernel_info: KernelInfo,
}

impl<T: PhysicalMemory, V: VAT> OperatingSystem for Kernel<T, V> {}

impl<T: PhysicalMemory, V: VAT> Kernel<T, V> {
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
            self.kernel_info.start_block.dtb,
            &mut self.vat,
        );

        let mut eprocs = Vec::new();

        let list_start = self.kernel_info.eprocess_base + self.offsets.eproc_link;
        let mut list_entry = list_start;
        //let mut next_list_entry = reader.virt_read_addr(list_start + offsets.list_blink)?;

        loop {
            let eprocess = list_entry - self.offsets.eproc_link;
            eprocs.push(eprocess);

            // read next list entry
            list_entry = reader.virt_read_addr(list_entry + self.offsets.list_blink)?;
            if list_entry.is_null() || list_entry == list_start {
                break;
            }
        }

        Ok(eprocs)
    }

    pub fn ntoskrnl_process_info(&mut self) -> Result<Win32ProcessInfo> {
        // TODO: create a VirtualFromPhysical constructor for kernel_info
        let mut reader = VirtualFromPhysical::with_vat(
            &mut self.phys_mem,
            self.kernel_info.start_block.arch,
            self.kernel_info.start_block.arch,
            self.kernel_info.start_block.dtb,
            &mut self.vat,
        );

        // TODO: cache this header on creation :)
        // read pe header
        let mut pe_buf = vec![0; self.kernel_info.kernel_size.as_usize()];
        reader.virt_read_raw_into(self.kernel_info.kernel_base, &mut pe_buf)?;

        let pe = PeView::from_bytes(&pe_buf)?;

        // find PsActiveProcessHead
        let loaded_module_list = match pe.get_export_by_name("PsLoadedModuleList")? {
            Export::Symbol(s) => self.kernel_info.kernel_base + Length::from(*s),
            Export::Forward(_) => {
                return Err(Error::new(
                    "PsLoadedModuleList found but it was a forwarded export",
                ))
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
                _ => return Err(Error::new("invalid architecture")),
            };
        trace!("ldr_data_base_offs={:x}", ldr_data_base_offs);
        trace!("ldr_data_size_offs={:x}", ldr_data_size_offs);
        trace!("ldr_data_name_offs={:x}", ldr_data_name_offs);

        Ok(Win32ProcessInfo {
            address: self.kernel_info.kernel_base,

            pid: 0,
            name: "ntoskrnl.exe".to_string(),
            dtb: self.kernel_info.start_block.dtb,
            wow64: Address::null(),

            peb: Address::null(),
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
            self.kernel_info.start_block.dtb,
            &mut self.vat,
        );

        let pid: i32 = reader.virt_read(eprocess + self.offsets.eproc_pid)?;
        trace!("pid={}", pid);

        let name = reader.virt_read_cstr(eprocess + self.offsets.eproc_name, Length::from(16))?;
        trace!("name={}", name);

        let dtb = reader.virt_read_addr(eprocess + self.offsets.kproc_dtb)?;
        trace!("dtb={:x}", dtb);

        let wow64 = if self.offsets.eproc_wow64.is_zero() {
            trace!("eproc_wow64=null; skipping wow64 detection");
            Address::null()
        } else {
            trace!(
                "eproc_wow64=${:x}; trying to read wow64 pointer",
                self.offsets.eproc_wow64
            );
            reader.virt_read_addr(eprocess + self.offsets.eproc_wow64)?
        };
        trace!("wow64={:x}", wow64);

        // read peb
        let peb = if wow64.is_null() {
            trace!("reading peb for native process");
            reader.virt_read_addr(eprocess + self.offsets.eproc_peb)?
        } else {
            trace!("reading peb for wow64 process");
            reader.virt_read_addr(wow64)?
        };
        trace!("peb={:x}", peb);

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
            _ => return Err(Error::new("invalid architecture")),
        };
        trace!("proc_arch={:?}", proc_arch);

        // from here on out we are in the process context
        // we will be using the process type architecture now
        let (peb_ldr_offs, ldr_list_offs) = match proc_arch.bits() {
            64 => (self.offsets.peb_ldr_x64, self.offsets.ldr_list_x64),
            32 => (self.offsets.peb_ldr_x86, self.offsets.ldr_list_x86),
            _ => return Err(Error::new("invalid architecture")),
        };
        trace!("peb_ldr_offs={:x}", peb_ldr_offs);
        trace!("ldr_list_offs={:x}", ldr_list_offs);

        // construct reader with process dtb
        // TODO: this wont work with tlb
        let mut proc_reader = VirtualFromPhysical::new(
            &mut self.phys_mem,
            self.kernel_info.start_block.arch,
            proc_arch,
            dtb,
        );
        let peb_ldr = proc_reader.virt_read_addr(peb + peb_ldr_offs)?;
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
            _ => return Err(Error::new("invalid architecture")),
        };
        trace!("ldr_data_base_offs={:x}", ldr_data_base_offs);
        trace!("ldr_data_size_offs={:x}", ldr_data_size_offs);
        trace!("ldr_data_name_offs={:x}", ldr_data_name_offs);

        Ok(Win32ProcessInfo {
            address: eprocess,

            pid,
            name,
            dtb,
            wow64,

            peb,
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
                process.name().to_lowercase() == name[..name.len().min(15)].to_lowercase()
            })
            .collect::<Vec<_>>();

        for &candidate in candidates.iter() {
            // TODO: properly probe pe header here and check ImageBase
            // TODO: this wont work with tlb
            let mut process = Win32Process::with_physical(self, candidate.clone());
            if let Ok(_) = process
                .module_info_list()?
                .iter()
                .inspect(|&module| println!("{:x} {}", module.base(), module.name()))
                .find(|&module| module.name().to_lowercase() == name.to_lowercase())
                .ok_or_else(|| Error::new(format!("unable to find module {}", name)))
            {
                return Ok(candidate.clone());
            }
        }

        Err(Error::new(format!("unable to find process {}", name)))
    }
}

impl<T: PhysicalMemory, V: VAT> fmt::Debug for Kernel<T, V> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self.kernel_info)
    }
}
