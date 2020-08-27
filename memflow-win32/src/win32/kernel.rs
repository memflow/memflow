use std::prelude::v1::*;

use super::{
    KernelBuilder, KernelInfo, Win32ExitStatus, Win32Process, Win32ProcessInfo,
    EXIT_STATUS_STILL_ACTIVE,
};
use crate::error::{Error, Result};
use crate::offsets::{Win32ArchOffsets, Win32Offsets};
use crate::pe::{pe32, pe64, MemoryPeViewContext};

use log::{info, trace};
use std::fmt;

use memflow_core::architecture::x86;
use memflow_core::mem::{
    DirectTranslate, PhysicalMemory, VirtualDMA, VirtualMemory, VirtualTranslate,
};
use memflow_core::process::{OperatingSystem, OsProcessInfo, OsProcessModuleInfo, PID};
use memflow_core::types::Address;

use super::Win32VirtualTranslate;

use pelite::{
    self,
    pe32::exports::GetProcAddress as GetProcAddress32,
    pe64::exports::{Export, GetProcAddress},
};

const MAX_ITER_COUNT: usize = 65536;

#[derive(Clone)]
pub struct Kernel<T, V> {
    pub phys_mem: T,
    pub vat: V,
    pub offsets: Win32Offsets,

    pub kernel_info: KernelInfo,
    pub sysproc_dtb: Address,
}

impl<T: PhysicalMemory, V: VirtualTranslate> OperatingSystem for Kernel<T, V> {}

impl<T: PhysicalMemory, V: VirtualTranslate> Kernel<T, V> {
    pub fn new(
        mut phys_mem: T,
        mut vat: V,
        offsets: Win32Offsets,
        kernel_info: KernelInfo,
    ) -> Self {
        // start_block only contains the winload's dtb which might
        // be different to the one used in the actual kernel.
        // In case of a failure this will fall back to the winload dtb.
        let sysproc_dtb = {
            let mut reader = VirtualDMA::with_vat(
                &mut phys_mem,
                kernel_info.start_block.arch,
                Win32VirtualTranslate::new(
                    kernel_info.start_block.arch,
                    kernel_info.start_block.dtb,
                ),
                &mut vat,
            );

            if let Ok(dtb) = reader.virt_read_addr_arch(
                kernel_info.start_block.arch,
                kernel_info.eprocess_base + offsets.kproc_dtb(),
            ) {
                dtb
            } else {
                kernel_info.start_block.dtb
            }
        };
        info!("sysproc_dtb={:x}", sysproc_dtb);

        Self {
            phys_mem,
            vat,
            offsets,

            kernel_info,
            sysproc_dtb,
        }
    }

    /// Consume the self object and return the containing memory connection
    pub fn destroy(self) -> T {
        self.phys_mem
    }

    pub fn eprocess_list(&mut self) -> Result<Vec<Address>> {
        // TODO: create a VirtualDMA constructor for kernel_info
        let mut reader = VirtualDMA::with_vat(
            &mut self.phys_mem,
            self.kernel_info.start_block.arch,
            Win32VirtualTranslate::new(self.kernel_info.start_block.arch, self.sysproc_dtb),
            &mut self.vat,
        );

        let mut eprocs = Vec::new();

        let list_start = self.kernel_info.eprocess_base + self.offsets.eproc_link();
        let mut list_entry = list_start;

        for _ in 0..MAX_ITER_COUNT {
            let eprocess = list_entry - self.offsets.eproc_link();
            trace!("eprocess={}", eprocess);

            // test flink + blink before adding the process
            let flink_entry =
                reader.virt_read_addr_arch(self.kernel_info.start_block.arch, list_entry)?;
            trace!("flink_entry={}", flink_entry);
            let blink_entry = reader.virt_read_addr_arch(
                self.kernel_info.start_block.arch,
                list_entry + self.offsets.list_blink(),
            )?;
            trace!("blink_entry={}", blink_entry);

            if flink_entry.is_null()
                || blink_entry.is_null()
                || flink_entry == list_start
                || flink_entry == list_entry
            {
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
        // TODO: create a VirtualDMA constructor for kernel_info
        let mut reader = VirtualDMA::with_vat(
            &mut self.phys_mem,
            self.kernel_info.start_block.arch,
            Win32VirtualTranslate::new(self.kernel_info.start_block.arch, self.sysproc_dtb),
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

        let peb_module =
            reader.virt_read_addr_arch(self.kernel_info.start_block.arch, loaded_module_list)?;

        // determine the offsets to be used when working with this process
        let (ldr_data_base_offs, ldr_data_size_offs, ldr_data_name_offs) = {
            let offsets = Win32ArchOffsets::from(self.kernel_info.start_block.arch);
            (
                offsets.ldr_data_base,
                offsets.ldr_data_size,
                offsets.ldr_data_name,
            )
        };

        trace!("ldr_data_base_offs={:x}", ldr_data_base_offs);
        trace!("ldr_data_size_offs={:x}", ldr_data_size_offs);
        trace!("ldr_data_name_offs={:x}", ldr_data_name_offs);

        Ok(Win32ProcessInfo {
            address: self.kernel_info.kernel_base,

            pid: 0,
            name: "ntoskrnl.exe".to_string(),
            dtb: self.sysproc_dtb,
            section_base: Address::NULL, // TODO: see below
            exit_status: EXIT_STATUS_STILL_ACTIVE,
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
        // TODO: create a VirtualDMA constructor for kernel_info
        let mut reader = VirtualDMA::with_vat(
            &mut self.phys_mem,
            self.kernel_info.start_block.arch,
            Win32VirtualTranslate::new(self.kernel_info.start_block.arch, self.sysproc_dtb),
            &mut self.vat,
        );

        let pid: PID = reader.virt_read(eprocess + self.offsets.eproc_pid())?;
        trace!("pid={}", pid);

        let name = reader.virt_read_cstr(eprocess + self.offsets.eproc_name(), 16)?;
        trace!("name={}", name);

        let dtb = reader.virt_read_addr_arch(
            self.kernel_info.start_block.arch,
            eprocess + self.offsets.kproc_dtb(),
        )?;
        trace!("dtb={:x}", dtb);

        let wow64 = if self.offsets.eproc_wow64() == 0 {
            trace!("eproc_wow64=null; skipping wow64 detection");
            Address::null()
        } else {
            trace!(
                "eproc_wow64={:x}; trying to read wow64 pointer",
                self.offsets.eproc_wow64()
            );
            reader.virt_read_addr_arch(
                self.kernel_info.start_block.arch,
                eprocess + self.offsets.eproc_wow64(),
            )?
        };
        trace!("wow64={:x}", wow64);

        // determine process architecture
        let sys_arch = self.kernel_info.start_block.arch;
        trace!("sys_arch={:?}", sys_arch);
        let proc_arch = match sys_arch.bits() {
            64 => {
                if wow64.is_null() {
                    x86::x64::ARCH
                } else {
                    x86::x32::ARCH
                }
            }
            32 => x86::x32::ARCH,
            _ => return Err(Error::InvalidArchitecture),
        };
        trace!("proc_arch={:?}", proc_arch);

        // read native_peb (either the process peb or the peb containing the wow64 helpers)
        let native_peb = reader.virt_read_addr_arch(
            self.kernel_info.start_block.arch,
            eprocess + self.offsets.eproc_peb(),
        )?;
        trace!("native_peb={:x}", native_peb);

        let section_base = reader.virt_read_addr_arch(
            self.kernel_info.start_block.arch,
            eprocess + self.offsets.eproc_section_base(),
        )?;
        trace!("section_base={:x}", section_base);

        let exit_status: Win32ExitStatus =
            reader.virt_read(eprocess + self.offsets.eproc_exit_status())?;
        trace!("exit_status={}", exit_status);

        // find first ethread
        let ethread = reader.virt_read_addr_arch(
            self.kernel_info.start_block.arch,
            eprocess + self.offsets.eproc_thread_list(),
        )? - self.offsets.ethread_list_entry();
        trace!("ethread={:x}", ethread);

        // TODO: does this need to be read with the process ctx?
        let teb = if wow64.is_null() {
            reader.virt_read_addr_arch(
                self.kernel_info.start_block.arch,
                ethread + self.offsets.kthread_teb(),
            )?
        } else {
            reader.virt_read_addr_arch(
                self.kernel_info.start_block.arch,
                ethread + self.offsets.kthread_teb(),
            )? + 0x2000
        };
        trace!("teb={:x}", teb);

        std::mem::drop(reader);

        // construct reader with process dtb
        // TODO: can tlb be used here already?
        let mut proc_reader = VirtualDMA::with_vat(
            &mut self.phys_mem,
            proc_arch,
            Win32VirtualTranslate::new(self.kernel_info.start_block.arch, dtb),
            DirectTranslate::new(),
        );

        // from here on out we are in the process context
        // we will be using the process type architecture now
        let teb_peb = if wow64.is_null() {
            proc_reader.virt_read_addr_arch(
                self.kernel_info.start_block.arch,
                teb + self.offsets.teb_peb(),
            )?
        } else {
            proc_reader.virt_read_addr_arch(
                self.kernel_info.start_block.arch,
                teb + self.offsets.teb_peb_x86(),
            )?
        };
        trace!("teb_peb={:x}", teb_peb);

        let real_peb = if !teb_peb.is_null() {
            teb_peb
        } else {
            proc_reader.virt_read_addr_arch(
                self.kernel_info.start_block.arch,
                eprocess + self.offsets.eproc_peb(),
            )?
        };
        trace!("real_peb={:x}", real_peb);

        let proc_offs = Win32ArchOffsets::from(proc_arch);

        trace!("peb_ldr_offs={:x}", proc_offs.peb_ldr);
        trace!("ldr_list_offs={:x}", proc_offs.ldr_list);

        let peb_ldr = proc_reader.virt_read_addr_arch(
            self.kernel_info.start_block.arch,
            real_peb /* TODO: can we have both? */ + proc_offs.peb_ldr,
        )?;
        trace!("peb_ldr={:x}", peb_ldr);

        let peb_module = proc_reader.virt_read_addr_arch(
            self.kernel_info.start_block.arch,
            peb_ldr + proc_offs.ldr_list,
        )?;
        trace!("peb_module={:x}", peb_module);

        let (ldr_data_base_offs, ldr_data_size_offs, ldr_data_name_offs) = (
            proc_offs.ldr_data_base,
            proc_offs.ldr_data_size,
            proc_offs.ldr_data_name,
        );

        trace!("ldr_data_base_offs={:x}", ldr_data_base_offs);
        trace!("ldr_data_size_offs={:x}", ldr_data_size_offs);
        trace!("ldr_data_name_offs={:x}", ldr_data_name_offs);

        Ok(Win32ProcessInfo {
            address: eprocess,

            pid,
            name,
            dtb,
            section_base,
            exit_status,
            ethread,
            wow64,

            teb,

            peb: real_peb, // TODO: store native + real peb - the wow64 Peb could be made an Option<>
            peb_module,

            sys_arch,
            proc_arch,

            ldr_data_base_offs,
            ldr_data_size_offs,
            ldr_data_name_offs,
        })
    }

    /// Retrieves a list of `Win32ProcessInfo` structs for all processes
    /// that can be found on the target system.
    pub fn process_info_list(&mut self) -> Result<Vec<Win32ProcessInfo>> {
        let mut list = Vec::new();
        for &eprocess in self.eprocess_list()?.iter() {
            if let Ok(prc) = self.process_info_from_eprocess(eprocess) {
                list.push(prc);
            }
        }
        Ok(list)
    }

    /// Finds a process by it's name and returns the `Win32ProcessInfo` struct.
    /// If no process with the specified name can be found this function will return an Error.
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
            let mut process = Win32Process::with_kernel_ref(self, candidate.clone());
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

    /// Finds a process by it's process id and returns the `Win32ProcessInfo` struct.
    /// If no process with the specified name can be found this function will return an Error.
    pub fn process_info_pid(&mut self, pid: PID) -> Result<Win32ProcessInfo> {
        let process_info_list = self.process_info_list()?;
        process_info_list
            .into_iter()
            .inspect(|process| trace!("{} {}", process.pid(), process.name()))
            .find(|process| process.pid == pid)
            .ok_or_else(|| Error::Other("pid not found"))
    }

    /// Finds a process by its name and constructs a `Win32Process` struct
    /// by borrowing this kernel instance.
    /// If no process with the specified name can be found this function will return an Error.
    ///
    /// This function can be useful for quickly accessing a process.
    pub fn process(
        &mut self,
        name: &str,
    ) -> Result<Win32Process<VirtualDMA<&mut T, &mut V, Win32VirtualTranslate>>> {
        let proc_info = self.process_info(name)?;
        Ok(Win32Process::with_kernel_ref(self, proc_info))
    }

    /// Finds a process by its process id and constructs a `Win32Process` struct
    /// by borrowing this kernel instance.
    /// If no process with the specified name can be found this function will return an Error.
    ///
    /// This function can be useful for quickly accessing a process.
    pub fn process_pid(
        &mut self,
        pid: PID,
    ) -> Result<Win32Process<VirtualDMA<&mut T, &mut V, Win32VirtualTranslate>>> {
        let proc_info = self.process_info_pid(pid)?;
        Ok(Win32Process::with_kernel_ref(self, proc_info))
    }

    /// Finds a process by its name and constructs a `Win32Process` struct
    /// by consuming the kernel struct and moving it into the process.
    ///
    /// If necessary the kernel can be retrieved back by calling `destroy()` on the process again.
    ///
    /// If no process with the specified name can be found this function will return an Error.
    ///
    /// This function can be useful for quickly accessing a process.
    pub fn into_process(
        mut self,
        name: &str,
    ) -> Result<Win32Process<VirtualDMA<T, V, Win32VirtualTranslate>>> {
        let proc_info = self.process_info(name)?;
        Ok(Win32Process::with_kernel(self, proc_info))
    }

    /// Finds a process by its process id and constructs a `Win32Process` struct
    /// by consuming the kernel struct and moving it into the process.
    ///
    /// If necessary the kernel can be retrieved back by calling `destroy()` on the process again.
    ///
    /// If no process with the specified name can be found this function will return an Error.
    ///
    /// This function can be useful for quickly accessing a process.
    pub fn into_process_pid(
        mut self,
        pid: PID,
    ) -> Result<Win32Process<VirtualDMA<T, V, Win32VirtualTranslate>>> {
        let proc_info = self.process_info_pid(pid)?;
        Ok(Win32Process::with_kernel(self, proc_info))
    }
}

impl<T: PhysicalMemory> Kernel<T, DirectTranslate> {
    pub fn builder(connector: T) -> KernelBuilder<T, T, DirectTranslate> {
        KernelBuilder::<T, T, DirectTranslate>::new(connector)
    }
}

impl<T: PhysicalMemory, V: VirtualTranslate> fmt::Debug for Kernel<T, V> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self.kernel_info)
    }
}
