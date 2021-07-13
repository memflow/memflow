use crate::architecture::x86::x64;
use crate::architecture::VirtualTranslate3;
use crate::error::*;

use crate::architecture::ArchitectureIdent;
use crate::mem::{mem_data::*, memory_view::*};
use crate::os::process::*;
use crate::os::*;
use crate::types::Address;

use crate::cglue::*;
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
        let base = self.info.address + thread_rng().gen_range(0..((self.map_size - min_size) / 2));

        for i in 0..count {
            self.modules.push(ModuleInfo {
                address: Address::from(i * 1024),
                parent_process: Address::INVALID,
                base,
                size: (thread_rng().gen_range(min_size..(self.map_size - base.as_usize()))),
                name: "dummy.so".into(),
                path: "/".into(),
                arch: x64::ARCH.ident(),
            });
        }
    }

    pub fn translator(&self) -> impl VirtualTranslate3 {
        x64::new_translator(self.dtb)
    }
}

cglue_impl_group!(DummyProcess<T>, ProcessInstance, {});
cglue_impl_group!(DummyProcess<T>, IntoProcessInstance, {});

#[derive(Clone)]
pub struct DummyProcess<T> {
    pub proc: DummyProcessInfo,
    pub mem: T,
}

impl<T: MemoryView> Process for DummyProcess<T> {
    /// Retrieves virtual address translator for the process (if applicable)
    //fn vat(&mut self) -> Option<&mut Self::VirtualTranslateType>;

    fn state(&mut self) -> ProcessState {
        ProcessState::Alive
    }

    /// Walks the process' module list and calls the provided callback for each module
    fn module_address_list_callback(
        &mut self,
        target_arch: Option<&ArchitectureIdent>,
        callback: ModuleAddressCallback,
    ) -> Result<()> {
        self.proc
            .modules
            .iter()
            .filter_map(|m| {
                if target_arch.is_none() || Some(&m.arch) == target_arch {
                    Some(ModuleAddressInfo {
                        address: m.address,
                        arch: m.arch,
                    })
                } else {
                    None
                }
            })
            .feed_into(callback);
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

    fn module_import_list_callback(
        &mut self,
        _info: &ModuleInfo,
        _callback: ImportCallback,
    ) -> Result<()> {
        Err(Error(ErrorOrigin::OsLayer, ErrorKind::NotImplemented))
    }

    fn module_export_list_callback(
        &mut self,
        _info: &ModuleInfo,
        _callback: ExportCallback,
    ) -> Result<()> {
        Err(Error(ErrorOrigin::OsLayer, ErrorKind::NotImplemented))
    }

    fn module_section_list_callback(
        &mut self,
        _info: &ModuleInfo,
        _callback: SectionCallback,
    ) -> Result<()> {
        Err(Error(ErrorOrigin::OsLayer, ErrorKind::NotImplemented))
    }

    /// Retrieves the process info
    fn info(&self) -> &ProcessInfo {
        &self.proc.info
    }
}

impl<T: MemoryView> MemoryView for DummyProcess<T> {
    fn read_raw_iter<'a>(
        &mut self,
        data: CIterator<ReadData<'a>>,
        out_fail: &mut ReadFailCallback<'_, 'a>,
    ) -> Result<()> {
        self.mem.read_raw_iter(data, out_fail)
    }

    fn write_raw_iter<'a>(
        &mut self,
        data: CIterator<WriteData<'a>>,
        out_fail: &mut WriteFailCallback<'_, 'a>,
    ) -> Result<()> {
        self.mem.write_raw_iter(data, out_fail)
    }

    fn metadata(&self) -> MemoryViewMetadata {
        self.mem.metadata()
    }
}

#[cfg(test)]
mod tests {
    use super::super::*;
    use crate::os::{OsInner, Process};
    use crate::types::size;

    #[test]
    pub fn primary_module() {
        let mem = DummyMemory::new(size::mb(64));
        let mut os = DummyOs::new(mem);

        let pid = os.alloc_process(size::mb(60), &[]);
        let mut prc = os.process_by_pid(pid).unwrap();
        prc.proc.add_modules(10, size::kb(1));

        let module = prc.primary_module();
        assert!(module.is_ok())
    }
}
