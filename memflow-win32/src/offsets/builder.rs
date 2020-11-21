use std::convert::TryFrom;

#[cfg(feature = "symstore")]
use super::symstore::SymbolStore;

use super::offset_table::Win32OffsetFile;
use super::{Win32Offsets, Win32OffsetsArchitecture};

use crate::error::{Error, Result};
use crate::kernel::{Win32GUID, Win32Version};
use crate::win32::KernelInfo;

#[repr(align(16))]
struct Align16<T>(pub T);

#[cfg(feature = "embed_offsets")]
const WIN32_OFFSETS: Align16<
    [u8; include_bytes!(concat!(env!("OUT_DIR"), "/win32_offsets.bin")).len()],
> = Align16(*include_bytes!(concat!(
    env!("OUT_DIR"),
    "/win32_offsets.bin"
)));

pub struct Win32OffsetBuilder {
    #[cfg(feature = "symstore")]
    symbol_store: Option<SymbolStore>,

    guid: Option<Win32GUID>,
    winver: Option<Win32Version>,
    arch: Option<Win32OffsetsArchitecture>,
}

impl Default for Win32OffsetBuilder {
    fn default() -> Self {
        Self {
            #[cfg(feature = "symstore")]
            symbol_store: Some(SymbolStore::default()),

            guid: None,
            winver: None,
            arch: None,
        }
    }
}

impl Win32OffsetBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn build(self) -> Result<Win32Offsets> {
        if self.guid.is_none() && self.winver.is_none() {
            return Err(Error::Other(
                "building win32 offsets requires either a guid or winver",
            ));
        }

        // try to build via symbol store
        if let Ok(offs) = self.build_with_symbol_store() {
            return Ok(offs);
        }

        // use static offset list
        if let Ok(offs) = self.build_with_offset_list() {
            return Ok(offs);
        }

        Err(Error::Other("not found"))
    }

    #[cfg(feature = "embed_offsets")]
    fn build_with_offset_list(&self) -> Result<Win32Offsets> {
        // # Safety
        // Struct padding and alignment is compile-time guaranteed by the struct (see mod offset_table).
        let offsets: [Win32OffsetFile;
            WIN32_OFFSETS.0.len() / std::mem::size_of::<Win32OffsetFile>()] =
            unsafe { std::mem::transmute(WIN32_OFFSETS.0) };

        // Try matching exact guid
        if let Some(target_guid) = &self.guid {
            for offset in offsets.iter() {
                if let (Ok(file), Ok(guid)) = (
                    <&str>::try_from(&offset.pdb_file_name),
                    <&str>::try_from(&offset.pdb_guid),
                ) {
                    if target_guid.file_name == file && target_guid.guid == guid {
                        return Ok(Win32Offsets {
                            0: offset.offsets.clone(),
                        });
                    }
                }
            }
        }

        let mut closest_match = None;
        let mut prev_build_number = 0;

        // Try matching the newest build from that version that is not actually newer
        if let (Some(winver), Some(arch)) = (&self.winver, self.arch) {
            for offset in offsets.iter() {
                if winver.major_version() == offset.nt_major_version
                    && winver.minor_version() == offset.nt_minor_version
                    && winver.build_number() >= offset.nt_build_number
                    && prev_build_number <= offset.nt_build_number
                    && arch == offset.arch
                {
                    prev_build_number = offset.nt_build_number;
                    closest_match = Some(Win32Offsets {
                        0: offset.offsets.clone(),
                    });
                }
            }

            if prev_build_number != winver.build_number() {
                log::warn!(
                    "no exact build number ({}) found! Closest match: {}",
                    winver.build_number(),
                    prev_build_number
                );
            }
        }

        closest_match.ok_or(Error::Other("not found"))
    }

    #[cfg(not(feature = "embed_offsets"))]
    fn build_with_offset_list(&self) -> Result<Win32Offsets> {
        Err(Error::Other(
            "embed offsets feature is deactivated on compilation",
        ))
    }

    #[cfg(feature = "symstore")]
    fn build_with_symbol_store(&self) -> Result<Win32Offsets> {
        if let Some(store) = &self.symbol_store {
            if self.guid.is_some() {
                let pdb = store.load(self.guid.as_ref().unwrap())?;
                Win32Offsets::from_pdb_slice(&pdb[..])
            } else {
                Err(Error::Other("symbol store can only be used with a guid"))
            }
        } else {
            Err(Error::Other("symbol store is disabled"))
        }
    }

    #[cfg(not(feature = "symstore"))]
    fn build_with_symbol_store(&self) -> Result<Win32Offsets> {
        Err(Error::Other(
            "symbol store is deactivated via a compilation feature",
        ))
    }

    #[cfg(feature = "symstore")]
    pub fn symbol_store(mut self, symbol_store: SymbolStore) -> Self {
        self.symbol_store = Some(symbol_store);
        self
    }

    #[cfg(feature = "symstore")]
    pub fn no_symbol_store(mut self) -> Self {
        self.symbol_store = None;
        self
    }

    pub fn guid(mut self, guid: Win32GUID) -> Self {
        self.guid = Some(guid);
        self
    }

    pub fn winver(mut self, winver: Win32Version) -> Self {
        self.winver = Some(winver);
        self
    }

    pub fn arch(mut self, arch: Win32OffsetsArchitecture) -> Self {
        self.arch = Some(arch);
        self
    }

    pub fn kernel_info(mut self, kernel_info: &KernelInfo) -> Self {
        if self.guid.is_none() {
            self.guid = kernel_info.kernel_guid.clone();
        }
        if self.winver.is_none() {
            self.winver = Some(kernel_info.kernel_winver);
        }
        if self.arch.is_none() {
            self.arch = Some(kernel_info.start_block.arch.into());
        }
        self
    }
}
