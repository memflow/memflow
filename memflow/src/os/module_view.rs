//! Architecture-scoped module view over a process.

use super::{
    ExportCallback, ExportInfo, ImportCallback, ImportInfo, ModuleAddressCallback,
    ModuleAddressInfo, ModuleInfo, ModuleInfoCallback, Process, SectionCallback, SectionInfo,
};
use crate::architecture::ArchitectureIdent;
use crate::error::{Error, ErrorKind, ErrorOrigin};
use crate::prelude::v1::{Address, Result};

#[repr(C)]
#[derive(Clone, Debug)]
pub struct ModuleView<T> {
    process: T,
    target_arch: Option<ArchitectureIdent>,
}

impl<T> ModuleView<T> {
    #[inline]
    pub fn new(process: T, target_arch: Option<ArchitectureIdent>) -> Self {
        Self {
            process,
            target_arch,
        }
    }
}

impl<'a, P: Process + ?Sized> ModuleView<&'a mut P> {
    #[inline]
    pub fn process(&self) -> &P {
        &self.process
    }

    #[inline]
    pub fn process_mut(&mut self) -> &mut P {
        &mut self.process
    }

    #[inline]
    pub fn into_process(self) -> &'a mut P {
        self.process
    }

    #[inline]
    pub fn target_arch(&self) -> Option<ArchitectureIdent> {
        self.target_arch
    }

    #[inline]
    fn effective_target_arch(&self) -> ArchitectureIdent {
        self.target_arch.unwrap_or(self.process.info().proc_arch)
    }

    #[inline]
    pub fn module_address_list_callback(&mut self, callback: ModuleAddressCallback) -> Result<()> {
        let target_arch = self.effective_target_arch();
        self.process
            .module_address_list_callback(Some(&target_arch), callback)
    }

    pub fn module_list_callback(&mut self, mut callback: ModuleInfoCallback) -> Result<()> {
        let sptr = self.process as *mut P;
        let target_arch = self.effective_target_arch();
        let inner_callback = &mut |ModuleAddressInfo { address, arch }| match unsafe { &mut *sptr }
            .module_by_address(address, arch)
        {
            Ok(info) => callback.call(info),
            Err(e) => {
                log::trace!("Error when reading module {:x} {:?}", address, e);
                true
            }
        };

        self.process
            .module_address_list_callback(Some(&target_arch), inner_callback.into())
    }

    #[inline]
    pub fn module_by_address(&mut self, address: Address) -> Result<ModuleInfo> {
        self.process
            .module_by_address(address, self.effective_target_arch())
    }

    pub fn module_by_name(&mut self, name: &str) -> Result<ModuleInfo> {
        let mut ret = Err(Error(ErrorOrigin::OsLayer, ErrorKind::ModuleNotFound));
        let callback = &mut |data: ModuleInfo| {
            if data.name.as_ref() == name {
                ret = Ok(data);
                false
            } else {
                true
            }
        };
        self.module_list_callback(callback.into())?;
        ret
    }

    pub fn module_by_name_ignore_ascii_case(&mut self, name: &str) -> Result<ModuleInfo> {
        let mut ret = Err(Error(ErrorOrigin::OsLayer, ErrorKind::ModuleNotFound));
        let callback = &mut |data: ModuleInfo| {
            if data.name.as_ref().eq_ignore_ascii_case(name) {
                ret = Ok(data);
                false
            } else {
                true
            }
        };
        self.module_list_callback(callback.into())?;
        ret
    }

    pub fn module_list(&mut self) -> Result<Vec<ModuleInfo>> {
        let mut ret = vec![];
        self.module_list_callback((&mut ret).into())?;
        Ok(ret)
    }

    #[inline]
    pub fn primary_module_address(&mut self) -> Result<Address> {
        self.process
            .primary_module_address_arch(self.target_arch.as_ref())
    }

    #[inline]
    pub fn primary_module(&mut self) -> Result<ModuleInfo> {
        self.process.primary_module_arch(self.target_arch.as_ref())
    }

    #[inline]
    pub fn module_import_list_callback(
        &mut self,
        info: &ModuleInfo,
        callback: ImportCallback,
    ) -> Result<()> {
        self.process.module_import_list_callback(info, callback)
    }

    #[inline]
    pub fn module_export_list_callback(
        &mut self,
        info: &ModuleInfo,
        callback: ExportCallback,
    ) -> Result<()> {
        self.process.module_export_list_callback(info, callback)
    }

    #[inline]
    pub fn module_section_list_callback(
        &mut self,
        info: &ModuleInfo,
        callback: SectionCallback,
    ) -> Result<()> {
        self.process.module_section_list_callback(info, callback)
    }

    pub fn module_import_list(&mut self, info: &ModuleInfo) -> Result<Vec<ImportInfo>> {
        let mut ret = vec![];
        self.process
            .module_import_list_callback(info, (&mut ret).into())?;
        Ok(ret)
    }

    pub fn module_export_list(&mut self, info: &ModuleInfo) -> Result<Vec<ExportInfo>> {
        let mut ret = vec![];
        self.process
            .module_export_list_callback(info, (&mut ret).into())?;
        Ok(ret)
    }

    pub fn module_section_list(&mut self, info: &ModuleInfo) -> Result<Vec<SectionInfo>> {
        let mut ret = vec![];
        self.process
            .module_section_list_callback(info, (&mut ret).into())?;
        Ok(ret)
    }

    pub fn module_import_by_name(&mut self, info: &ModuleInfo, name: &str) -> Result<ImportInfo> {
        self.process.module_import_by_name(info, name)
    }

    pub fn module_export_by_name(&mut self, info: &ModuleInfo, name: &str) -> Result<ExportInfo> {
        self.process.module_export_by_name(info, name)
    }

    pub fn module_section_by_name(&mut self, info: &ModuleInfo, name: &str) -> Result<SectionInfo> {
        self.process.module_section_by_name(info, name)
    }
}
