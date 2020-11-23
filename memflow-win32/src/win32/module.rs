use std::prelude::v1::*;

use memflow::process::OsProcessModuleInfo;
use memflow::types::Address;

#[derive(Debug, Clone)]
pub struct Win32ModuleInfo {
    pub peb_entry: Address,
    pub parent_eprocess: Address, // parent "reference"

    pub base: Address, // _LDR_DATA_TABLE_ENTRY::DllBase
    pub size: usize,   // _LDR_DATA_TABLE_ENTRY::SizeOfImage
    pub path: String,  // _LDR_DATA_TABLE_ENTRY::FullDllName
    pub name: String,  // _LDR_DATA_TABLE_ENTRY::BaseDllName
}

impl OsProcessModuleInfo for Win32ModuleInfo {
    fn address(&self) -> Address {
        self.peb_entry
    }

    fn parent_process(&self) -> Address {
        self.parent_eprocess
    }

    fn base(&self) -> Address {
        self.base
    }

    fn size(&self) -> usize {
        self.size
    }

    fn name(&self) -> String {
        self.name.clone()
    }
}
