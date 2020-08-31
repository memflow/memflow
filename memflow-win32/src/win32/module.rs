use std::prelude::v1::*;

use memflow_core::process::OsProcessModuleInfo;
use memflow_core::types::Address;

#[derive(Debug, Clone)]
pub struct Win32ModuleInfo {
    pub peb_entry: Address,
    pub parent_eprocess: Address, // parent "reference"

    pub base: Address,
    pub size: usize,
    pub name: String,
    // exports
    // sections
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
