use std::prelude::v1::*;

use super::{
    process::EXIT_STATUS_STILL_ACTIVE, process::IMAGE_FILE_NAME_LENGTH, KernelBuilder, KernelInfo,
    Win32ExitStatus, Win32ModuleListInfo, Win32Process, Win32ProcessInfo, Win32VirtualTranslate,
};

use crate::error::{Error, Result};
use crate::offsets::Win32Offsets;

use log::{info, trace};
use std::fmt;

use memflow::architecture::x86;
use memflow::mem::{DirectTranslate, PhysicalMemory, VirtualDMA, VirtualMemory, VirtualTranslate};
use memflow::process::{OperatingSystem, OsProcessInfo, OsProcessModuleInfo, PID};
use memflow::types::Address;

use pelite::{self, pe64::exports::Export, PeView};

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
        let mut eprocs = Vec::new();
        self.eprocess_list_extend(&mut eprocs)?;
        trace!("found {} eprocesses", eprocs.len());
        Ok(eprocs)
    }

    pub fn eprocess_list_extend<E: Extend<Address>>(&mut self, eprocs: &mut E) -> Result<()> {
        // TODO: create a VirtualDMA constructor for kernel_info
        let mut reader = VirtualDMA::with_vat(
            &mut self.phys_mem,
            self.kernel_info.start_block.arch,
            Win32VirtualTranslate::new(self.kernel_info.start_block.arch, self.sysproc_dtb),
            &mut self.vat,
        );

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
            eprocs.extend(Some(eprocess).into_iter());

            // continue
            list_entry = flink_entry;
        }

        Ok(())
    }

    pub fn kernel_process_info(&mut self) -> Result<Win32ProcessInfo> {
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
            let image =
                reader.virt_read_raw(self.kernel_info.kernel_base, self.kernel_info.kernel_size)?;
            let pe = PeView::from_bytes(&image).map_err(Error::PE)?;
            match pe
                .get_export_by_name("PsLoadedModuleList")
                .map_err(Error::PE)?
            {
                Export::Symbol(s) => self.kernel_info.kernel_base + *s as usize,
                Export::Forward(_) => {
                    return Err(Error::Other(
                        "PsLoadedModuleList found but it was a forwarded export",
                    ))
                }
            }
        };

        let kernel_modules =
            reader.virt_read_addr_arch(self.kernel_info.start_block.arch, loaded_module_list)?;

        Ok(Win32ProcessInfo {
            address: self.kernel_info.kernel_base,

            pid: 0,
            name: "ntoskrnl.exe".to_string(),
            dtb: self.sysproc_dtb,
            section_base: Address::NULL, // TODO: see below
            exit_status: EXIT_STATUS_STILL_ACTIVE,
            ethread: Address::NULL, // TODO: see below
            wow64: Address::NULL,

            teb: None,
            teb_wow64: None,

            peb_native: Address::NULL,
            peb_wow64: None,

            module_info_native: Win32ModuleListInfo::with_base(
                kernel_modules,
                self.kernel_info.start_block.arch,
            )?,
            module_info_wow64: None,

            sys_arch: self.kernel_info.start_block.arch,
            proc_arch: self.kernel_info.start_block.arch,
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

        let name =
            reader.virt_read_cstr(eprocess + self.offsets.eproc_name(), IMAGE_FILE_NAME_LENGTH)?;
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

        let peb_native = reader
            .virt_read_addr_arch(
                self.kernel_info.start_block.arch,
                eprocess + self.offsets.eproc_peb(),
            )?
            .non_null()
            .ok_or(Error::Other("Could not retrieve peb_native"))?;

        let mut peb_wow64 = None;

        // TODO: does this need to be read with the process ctx?
        let (teb, teb_wow64) = if self.kernel_info.kernel_winver >= (6, 2).into() {
            let teb = reader.virt_read_addr_arch(
                self.kernel_info.start_block.arch,
                ethread + self.offsets.kthread_teb(),
            )?;

            trace!("teb={:x}", teb);

            if !teb.is_null() {
                (
                    Some(teb),
                    if wow64.is_null() {
                        None
                    } else {
                        Some(teb + 0x2000)
                    },
                )
            } else {
                (None, None)
            }
        } else {
            (None, None)
        };

        std::mem::drop(reader);

        // construct reader with process dtb
        // TODO: can tlb be used here already?
        let mut proc_reader = VirtualDMA::with_vat(
            &mut self.phys_mem,
            proc_arch,
            Win32VirtualTranslate::new(self.kernel_info.start_block.arch, dtb),
            DirectTranslate::new(),
        );

        if let Some(teb) = teb_wow64 {
            // from here on out we are in the process context
            // we will be using the process type architecture now
            peb_wow64 = proc_reader
                .virt_read_addr_arch(
                    self.kernel_info.start_block.arch,
                    teb + self.offsets.teb_peb_x86(),
                )?
                .non_null();

            trace!("peb_wow64={:?}", peb_wow64);
        }

        trace!("peb_native={:?}", peb_native);

        let module_info_native =
            Win32ModuleListInfo::with_peb(&mut proc_reader, peb_native, sys_arch)?;

        let module_info_wow64 = peb_wow64
            .map(|peb| Win32ModuleListInfo::with_peb(&mut proc_reader, peb, proc_arch))
            .transpose()?;

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
            teb_wow64,

            peb_native,
            peb_wow64,

            module_info_native,
            module_info_wow64,

            sys_arch,
            proc_arch,
        })
    }

    pub fn process_info_list_extend<E: Extend<Win32ProcessInfo>>(
        &mut self,
        list: &mut E,
    ) -> Result<()> {
        let mut vec = Vec::new();
        self.eprocess_list_extend(&mut vec)?;
        for eprocess in vec.into_iter() {
            if let Ok(prc) = self.process_info_from_eprocess(eprocess) {
                list.extend(Some(prc).into_iter());
            }
        }
        Ok(())
    }

    /// Retrieves a list of `Win32ProcessInfo` structs for all processes
    /// that can be found on the target system.
    pub fn process_info_list(&mut self) -> Result<Vec<Win32ProcessInfo>> {
        let mut list = Vec::new();
        self.process_info_list_extend(&mut list)?;
        Ok(list)
    }

    /// Finds a process by it's name and returns the `Win32ProcessInfo` struct.
    /// If no process with the specified name can be found this function will return an Error.
    pub fn process_info(&mut self, name: &str) -> Result<Win32ProcessInfo> {
        let name16 = name[..name.len().min(IMAGE_FILE_NAME_LENGTH - 1)].to_lowercase();

        let process_info_list = self.process_info_list()?;
        let candidates = process_info_list
            .iter()
            .inspect(|process| trace!("{} {}", process.pid(), process.name()))
            .filter(|process| {
                // strip process name to IMAGE_FILE_NAME_LENGTH without trailing \0
                process.name().to_lowercase() == name16
            })
            .collect::<Vec<_>>();

        for &candidate in candidates.iter() {
            // TODO: properly probe pe header here and check ImageBase
            // TODO: this wont work with tlb
            trace!("inspecting candidate process: {:?}", candidate);
            let mut process = Win32Process::with_kernel_ref(self, candidate.clone());
            if process
                .module_list()?
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
    /// If no process with the specified PID can be found this function will return an Error.
    ///
    /// If the specified PID is 0 the kernel process is returned.
    pub fn process_info_pid(&mut self, pid: PID) -> Result<Win32ProcessInfo> {
        if pid > 0 {
            // regular pid
            let process_info_list = self.process_info_list()?;
            process_info_list
                .into_iter()
                .inspect(|process| trace!("{} {}", process.pid(), process.name()))
                .find(|process| process.pid == pid)
                .ok_or_else(|| Error::Other("pid not found"))
        } else {
            // kernel pid
            self.kernel_process_info()
        }
    }

    /// Constructs a `Win32Process` struct for the targets kernel by borrowing this kernel instance.
    ///
    /// This function can be useful for quickly accessing the kernel process.
    pub fn kernel_process(
        &mut self,
    ) -> Result<Win32Process<VirtualDMA<&mut T, &mut V, Win32VirtualTranslate>>> {
        let proc_info = self.kernel_process_info()?;
        Ok(Win32Process::with_kernel_ref(self, proc_info))
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

    /// Constructs a `Win32Process` struct by consuming this kernel struct
    /// and moving it into the resulting process.
    ///
    /// If necessary the kernel can be retrieved back by calling `destroy()` on the process after use.
    ///
    /// This function can be useful for quickly accessing a process.
    pub fn into_kernel_process(
        mut self,
    ) -> Result<Win32Process<VirtualDMA<T, V, Win32VirtualTranslate>>> {
        let proc_info = self.kernel_process_info()?;
        Ok(Win32Process::with_kernel(self, proc_info))
    }

    /// Finds a process by its name and constructs a `Win32Process` struct
    /// by consuming the kernel struct and moving it into the process.
    ///
    /// If necessary the kernel can be retrieved back by calling `destroy()` on the process after use.
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
