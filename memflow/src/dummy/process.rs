use crate::architecture::x86::x64;
use crate::architecture::ScopedVirtualTranslate;
use crate::error::{Error, ErrorKind, ErrorOrigin, Result};

use crate::architecture::ArchitectureIdent;
use crate::mem::VirtualMemory;
use crate::os::{ModuleAddressCallback, ModuleAddressInfo, ModuleInfo, Process, ProcessInfo};
use crate::types::Address;

use rand::{thread_rng, Rng};

#[derive(Clone)]
pub struct DummyProcessInfo {
    pub info: ProcessInfo,
    pub map_size: usize,
    pub dtb: Address,
    pub modules: Vec<ModuleInfo>,
}

impl DummyProcessInfo {
    pub fn add_modules(&mut self, count: usize, min_size: usize) {
        for i in 0..count {
            self.modules.push(ModuleInfo {
                address: Address::from(i * 1024),
                parent_process: Address::INVALID,
                base: self.info.address + thread_rng().gen_range(0..self.map_size / 2),
                size: (thread_rng().gen_range(min_size..self.map_size) / 2),
                name: "dummy.so".into(),
                path: "/".into(),
                arch: x64::ARCH.ident(),
            });
        }
    }

    pub fn translator(&self) -> impl ScopedVirtualTranslate {
        x64::new_translator(self.dtb)
    }
}

#[derive(Clone)]
pub struct DummyProcess<T> {
    pub proc: DummyProcessInfo,
    pub mem: T,
}

impl<T: VirtualMemory> Process for DummyProcess<T> {
    type VirtualMemoryType = T;
    //type VirtualTranslateType: VirtualTranslate;

    /// Retrieves virtual memory object for the process
    fn virt_mem(&mut self) -> &mut Self::VirtualMemoryType {
        &mut self.mem
    }

    /// Retrieves virtual address translator for the process (if applicable)
    //fn vat(&mut self) -> Option<&mut Self::VirtualTranslateType>;

    /// Walks the process' module list and calls the provided callback for each module
    fn module_address_list_callback(
        &mut self,
        target_arch: Option<&ArchitectureIdent>,
        mut callback: ModuleAddressCallback,
    ) -> Result<()> {
        for m in self.proc.modules.iter() {
            if Some(&m.arch) == target_arch
                && !callback.call(ModuleAddressInfo {
                    address: m.address,
                    arch: m.arch,
                })
            {
                break;
            }
        }
        Ok(())
    }

    /// Retrieves a module by its structure address and architecture
    ///
    /// # Arguments
    /// * `address` - address where module's information resides in
    /// * `architecture` - architecture of the module. Should be either `ProcessInfo::proc_arch`, or `ProcessInfo::sys_arch`.
    fn module_by_address(
        &mut self,
        address: Address,
        architecture: ArchitectureIdent,
    ) -> Result<ModuleInfo> {
        self.proc
            .modules
            .iter()
            .filter(|m| m.address == address)
            .filter(|m| m.arch == architecture)
            .cloned()
            .next()
            .ok_or(Error(ErrorOrigin::OsLayer, ErrorKind::ModuleNotFound))
    }

    /// Retrieves address of the primary module structure of the process
    fn primary_module_address(&mut self) -> Result<Address> {
        let mut ret = Err(Error(ErrorOrigin::OsLayer, ErrorKind::ModuleNotFound));
        let callback = &mut |moduleinfo: ModuleAddressInfo| {
            ret = Ok(moduleinfo.address);
            false
        };
        let proc_arch = self.info().proc_arch;
        self.module_address_list_callback(Some(&proc_arch), callback.into())?;
        ret
    }

    /// Retrieves the process info
    fn info(&self) -> &ProcessInfo {
        &self.proc.info
    }
}
