pub mod pdb_struct;
pub use pdb_struct::PdbStruct;

pub mod pdb_cache;

use std::path::Path;

use crate::error::{Error, Result};
use crate::kernel::ntos::Win32GUID;

use flow_core::address::Length;

#[derive(Debug, Clone)]
pub struct Win32Offsets {
    pub blink: Length,
    pub eproc_link: Length,

    pub kproc_dtb: Length,
    pub eproc_pid: Length,
    pub eproc_name: Length,
    pub eproc_peb: Length,
    pub eproc_wow64: Length,
}

// initialize from pdb -> open pdb by file / by guid
// initialize from guid
// initialize manually

impl Win32Offsets {
    pub fn try_with_pdb(pdb_path: &Path) -> Result<Self> {
        let list = PdbStruct::with(pdb_path, "_LIST_ENTRY")?;
        let kproc = PdbStruct::with(pdb_path, "_KPROCESS")?;
        let eproc = PdbStruct::with(pdb_path, "_EPROCESS")?;

        let blink = list
            .find_field("Blink")
            .ok_or_else(|| Error::new("_LIST_ENTRY::Blink not found"))?
            .offset;

        let eproc_link = eproc
            .find_field("ActiveProcessLinks")
            .ok_or_else(|| Error::new("_EPROCESS::ActiveProcessLinks not found"))?
            .offset;

        let kproc_dtb = kproc
            .find_field("DirectoryTableBase")
            .ok_or_else(|| Error::new("_KPROCESS::DirectoryTableBase not found"))?
            .offset;
        let eproc_pid = eproc
            .find_field("UniqueProcessId")
            .ok_or_else(|| Error::new("_EPROCESS::UniqueProcessId not found"))?
            .offset;
        let eproc_name = eproc
            .find_field("ImageFileName")
            .ok_or_else(|| Error::new("_EPROCESS::ImageFileName not found"))?
            .offset;
        let eproc_peb = eproc
            .find_field("Peb")
            .ok_or_else(|| Error::new("_EPROCESS::Peb not found"))?
            .offset;
        let eproc_wow64 = match eproc.find_field("WoW64Process") {
            Some(f) => f.offset,
            None => Length::zero(),
        };

        Ok(Self {
            blink,
            eproc_link,
            kproc_dtb,
            eproc_pid,
            eproc_name,
            eproc_peb,
            eproc_wow64,
        })
    }

    pub fn try_with_guid(guid: &Win32GUID) -> Result<Self> {
        let pdb = pdb_cache::try_get_pdb(guid)?;
        Self::try_with_pdb(&pdb)
    }
}
