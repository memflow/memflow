pub mod builder;
pub use builder::Win32OffsetBuilder;

#[cfg(feature = "symstore")]
pub mod pdb_struct;
#[cfg(feature = "symstore")]
pub mod symstore;

pub mod offset_table;
#[doc(hidden)]
pub use offset_table::{Win32OffsetFile, Win32OffsetTable, Win32OffsetsArchitecture};

#[cfg(feature = "symstore")]
pub use {pdb_struct::PdbStruct, symstore::*};

use std::prelude::v1::*;

#[cfg(feature = "std")]
use std::{fs::File, io::Read, path::Path};

use crate::error::{Error, Result};
use crate::kernel::Win32GUID;
use memflow::architecture::{self, ArchitectureObj};

#[derive(Debug, Copy, Clone)]
#[repr(C)]
#[cfg_attr(feature = "serde", derive(::serde::Serialize))]
pub struct Win32ArchOffsets {
    pub peb_ldr: usize,            // _PEB::Ldr
    pub ldr_list: usize,           // _PEB_LDR_DATA::InLoadOrderModuleList
    pub ldr_data_base: usize,      // _LDR_DATA_TABLE_ENTRY::DllBase
    pub ldr_data_size: usize,      // _LDR_DATA_TABLE_ENTRY::SizeOfImage
    pub ldr_data_full_name: usize, // _LDR_DATA_TABLE_ENTRY::FullDllName
    pub ldr_data_base_name: usize, // _LDR_DATA_TABLE_ENTRY::BaseDllName
}

pub const X86: Win32ArchOffsets = Win32ArchOffsets {
    peb_ldr: 0xc,
    ldr_list: 0xc,
    ldr_data_base: 0x18,
    ldr_data_size: 0x20,
    ldr_data_full_name: 0x24,
    ldr_data_base_name: 0x2c,
};

pub const X64: Win32ArchOffsets = Win32ArchOffsets {
    peb_ldr: 0x18,
    ldr_list: 0x10,
    ldr_data_base: 0x30,
    ldr_data_size: 0x40,
    ldr_data_full_name: 0x48,
    ldr_data_base_name: 0x58,
};

impl Win32OffsetsArchitecture {
    #[inline]
    fn offsets(&self) -> &'static Win32ArchOffsets {
        match self {
            Win32OffsetsArchitecture::X64 => &X64,
            Win32OffsetsArchitecture::X86 => &X86,
            Win32OffsetsArchitecture::AArch64 => panic!("Not implemented"),
        }
    }
}

impl From<ArchitectureObj> for Win32ArchOffsets {
    fn from(arch: ArchitectureObj) -> Win32ArchOffsets {
        *Win32OffsetsArchitecture::from(arch).offsets()
    }
}

#[repr(transparent)]
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(::serde::Serialize))]
pub struct Win32Offsets(pub Win32OffsetTable);

impl From<Win32OffsetTable> for Win32Offsets {
    fn from(other: Win32OffsetTable) -> Self {
        Self { 0: other }
    }
}

impl From<Win32Offsets> for Win32OffsetTable {
    fn from(other: Win32Offsets) -> Self {
        other.0
    }
}

impl From<ArchitectureObj> for Win32OffsetsArchitecture {
    fn from(arch: ArchitectureObj) -> Win32OffsetsArchitecture {
        if arch == architecture::x86::x32::ARCH || arch == architecture::x86::x32_pae::ARCH {
            Self::X86
        } else if arch == architecture::x86::x64::ARCH {
            Self::X64
        } else {
            // We do not have AArch64, but that is in the plans...
            panic!("Invalid architecture specified")
        }
    }
}

impl Win32Offsets {
    #[cfg(feature = "symstore")]
    pub fn from_pdb<P: AsRef<Path>>(pdb_path: P) -> Result<Self> {
        let mut file = File::open(pdb_path)
            .map_err(|_| Error::PDB("unable to open user-supplied pdb file"))?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)
            .map_err(|_| Error::PDB("unable to read user-supplied pdb file"))?;
        Self::from_pdb_slice(&buffer[..])
    }

    #[cfg(feature = "symstore")]
    pub fn from_pdb_slice(pdb_slice: &[u8]) -> Result<Self> {
        let list = PdbStruct::with(pdb_slice, "_LIST_ENTRY")
            .map_err(|_| Error::PDB("_LIST_ENTRY not found"))?;
        let kproc = PdbStruct::with(pdb_slice, "_KPROCESS")
            .map_err(|_| Error::PDB("_KPROCESS not found"))?;
        let eproc = PdbStruct::with(pdb_slice, "_EPROCESS")
            .map_err(|_| Error::PDB("_EPROCESS not found"))?;
        let ethread =
            PdbStruct::with(pdb_slice, "_ETHREAD").map_err(|_| Error::PDB("_ETHREAD not found"))?;
        let kthread =
            PdbStruct::with(pdb_slice, "_KTHREAD").map_err(|_| Error::PDB("_KTHREAD not found"))?;
        let teb = PdbStruct::with(pdb_slice, "_TEB").map_err(|_| Error::PDB("_TEB not found"))?;

        let list_blink = list
            .find_field("Blink")
            .ok_or_else(|| Error::PDB("_LIST_ENTRY::Blink not found"))?
            .offset as _;

        let eproc_link = eproc
            .find_field("ActiveProcessLinks")
            .ok_or_else(|| Error::PDB("_EPROCESS::ActiveProcessLinks not found"))?
            .offset as _;

        let kproc_dtb = kproc
            .find_field("DirectoryTableBase")
            .ok_or_else(|| Error::PDB("_KPROCESS::DirectoryTableBase not found"))?
            .offset as _;
        let eproc_pid = eproc
            .find_field("UniqueProcessId")
            .ok_or_else(|| Error::PDB("_EPROCESS::UniqueProcessId not found"))?
            .offset as _;
        let eproc_name = eproc
            .find_field("ImageFileName")
            .ok_or_else(|| Error::PDB("_EPROCESS::ImageFileName not found"))?
            .offset as _;
        let eproc_peb = eproc
            .find_field("Peb")
            .ok_or_else(|| Error::PDB("_EPROCESS::Peb not found"))?
            .offset as _;
        let eproc_section_base = eproc
            .find_field("SectionBaseAddress")
            .ok_or_else(|| Error::PDB("_EPROCESS::SectionBaseAddress not found"))?
            .offset as _;
        let eproc_exit_status = eproc
            .find_field("ExitStatus")
            .ok_or_else(|| Error::PDB("_EPROCESS::ExitStatus not found"))?
            .offset as _;
        let eproc_thread_list = eproc
            .find_field("ThreadListHead")
            .ok_or_else(|| Error::PDB("_EPROCESS::ThreadListHead not found"))?
            .offset as _;

        // windows 10 uses an uppercase W whereas older windows versions (windows 7) uses a lowercase w
        let eproc_wow64 = match eproc
            .find_field("WoW64Process")
            .or_else(|| eproc.find_field("Wow64Process"))
        {
            Some(f) => f.offset as _,
            None => 0,
        };

        // threads
        let kthread_teb = kthread
            .find_field("Teb")
            .ok_or_else(|| Error::PDB("_KTHREAD::Teb not found"))?
            .offset as _;
        let ethread_list_entry = ethread
            .find_field("ThreadListEntry")
            .ok_or_else(|| Error::PDB("_ETHREAD::ThreadListEntry not found"))?
            .offset as _;
        let teb_peb = teb
            .find_field("ProcessEnvironmentBlock")
            .ok_or_else(|| Error::PDB("_TEB::ProcessEnvironmentBlock not found"))?
            .offset as _;
        let teb_peb_x86 = if let Ok(teb32) =
            PdbStruct::with(pdb_slice, "_TEB32").map_err(|_| Error::PDB("_TEB32 not found"))
        {
            teb32
                .find_field("ProcessEnvironmentBlock")
                .ok_or_else(|| Error::PDB("_TEB32::ProcessEnvironmentBlock not found"))?
                .offset as _
        } else {
            0
        };

        Ok(Self {
            0: Win32OffsetTable {
                list_blink,
                eproc_link,

                kproc_dtb,

                eproc_pid,
                eproc_name,
                eproc_peb,
                eproc_section_base,
                eproc_exit_status,
                eproc_thread_list,
                eproc_wow64,

                kthread_teb,
                ethread_list_entry,
                teb_peb,
                teb_peb_x86,
            },
        })
    }

    /// _LIST_ENTRY::Blink offset
    pub fn list_blink(&self) -> usize {
        self.0.list_blink as usize
    }
    /// _LIST_ENTRY::Flink offset
    pub fn eproc_link(&self) -> usize {
        self.0.eproc_link as usize
    }

    /// _KPROCESS::DirectoryTableBase offset
    /// Exists since version 3.10
    pub fn kproc_dtb(&self) -> usize {
        self.0.kproc_dtb as usize
    }
    /// _EPROCESS::UniqueProcessId offset
    /// Exists since version 3.10
    pub fn eproc_pid(&self) -> usize {
        self.0.eproc_pid as usize
    }
    /// _EPROCESS::ImageFileName offset
    /// Exists since version 3.10
    pub fn eproc_name(&self) -> usize {
        self.0.eproc_name as usize
    }
    /// _EPROCESS::Peb offset
    /// Exists since version 5.10
    pub fn eproc_peb(&self) -> usize {
        self.0.eproc_peb as usize
    }
    /// _EPROCESS::SectionBaseAddress offset
    /// Exists since version 3.10
    pub fn eproc_section_base(&self) -> usize {
        self.0.eproc_section_base as usize
    }
    /// _EPROCESS::ExitStatus offset
    /// Exists since version 3.10
    pub fn eproc_exit_status(&self) -> usize {
        self.0.eproc_exit_status as usize
    }
    /// _EPROCESS::ThreadListHead offset
    /// Exists since version 5.10
    pub fn eproc_thread_list(&self) -> usize {
        self.0.eproc_thread_list as usize
    }
    /// _EPROCESS::WoW64Process offset
    /// Exists since version 5.0
    pub fn eproc_wow64(&self) -> usize {
        self.0.eproc_wow64 as usize
    }

    /// _KTHREAD::Teb offset
    /// Exists since version 6.2
    pub fn kthread_teb(&self) -> usize {
        self.0.kthread_teb as usize
    }
    /// _ETHREAD::ThreadListEntry offset
    /// Exists since version 6.2
    pub fn ethread_list_entry(&self) -> usize {
        self.0.ethread_list_entry as usize
    }
    /// _TEB::ProcessEnvironmentBlock offset
    /// Exists since version x.x
    pub fn teb_peb(&self) -> usize {
        self.0.teb_peb as usize
    }
    /// _TEB32::ProcessEnvironmentBlock offset
    /// Exists since version x.x
    pub fn teb_peb_x86(&self) -> usize {
        self.0.teb_peb_x86 as usize
    }

    pub fn builder() -> Win32OffsetBuilder {
        Win32OffsetBuilder::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn download_pdb() {
        let guid = Win32GUID {
            file_name: "ntkrnlmp.pdb".to_string(),
            guid: "3844DBB920174967BE7AA4A2C20430FA2".to_string(),
        };
        let offsets = Win32Offsets::builder()
            .symbol_store(SymbolStore::new().no_cache())
            .guid(guid)
            .build()
            .unwrap();

        assert_eq!(offsets.0.list_blink, 8);
        assert_eq!(offsets.0.eproc_link, 392);

        assert_eq!(offsets.0.kproc_dtb, 40);

        assert_eq!(offsets.0.eproc_pid, 384);
        assert_eq!(offsets.0.eproc_name, 736);
        assert_eq!(offsets.0.eproc_peb, 824);
        assert_eq!(offsets.0.eproc_thread_list, 776);
        assert_eq!(offsets.0.eproc_wow64, 800);

        assert_eq!(offsets.0.kthread_teb, 184);
        assert_eq!(offsets.0.ethread_list_entry, 1056);
        assert_eq!(offsets.0.teb_peb, 96);
        assert_eq!(offsets.0.teb_peb_x86, 48);
    }
}
