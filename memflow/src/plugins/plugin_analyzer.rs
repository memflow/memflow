//! This module provides functionality to extract the PluginDescriptorInfo from a binary.
//!
//! Rather than loading the module in memory and finding the correct export
//! the analyzer will do a static analysis based on goblin to piece together
//! the PluginDescriptorInfo structure.

use std::ops::Range;

use dataview::{DataView, Pod};
use goblin::{
    elf::{header::ELFMAG, section_header::SHN_XINDEX, Elf},
    mach::{
        header::{MH_CIGAM, MH_CIGAM_64, MH_MAGIC, MH_MAGIC_64},
        Mach, MachO, SingleArch,
    },
    pe::{self, header::DOS_MAGIC, options::ParseOptions, PE},
    Object,
};
use num_traits::{NumCast, WrappingAdd, WrappingSub, Zero};
use serde::{Deserialize, Serialize};

use crate::{
    error::{Error, Result},
    plugins::{ErrorKind, ErrorOrigin},
};

const MEMFLOW_EXPORT_PREFIX_CONNECTOR: &str = "MEMFLOW_CONNECTOR_";
const MEMFLOW_EXPORT_PREFIX_OS: &str = "MEMFLOW_OS_";

/// The PluginDescriptorInfo struct is adapted and translated from memflow version 0.2.x:
/// https://github.com/memflow/memflow/blob/0.2.0/memflow/src/plugins/mod.rs#L105
#[repr(C, align(4))]
struct PluginDescriptorInfo32 {
    pub plugin_version: i32,
    pub accept_input: u8,   // bool
    pub input_layout: u32,  // &'static TypeLayout
    pub output_layout: u32, // &'static TypeLayout,
    pub name: u32,          // CSliceRef<'static, u8>,
    pub name_length: u32,
    pub version: u32, // CSliceRef<'static, u8>,
    pub version_length: u32,
    pub description: u32, //CSliceRef<'static, u8>,
    pub description_length: u32,
    pub help_callback: u32, // Option<extern "C" fn(callback: HelpCallback) -> ()>,
    pub target_list_callback: u32, // Option<extern "C" fn(callback: TargetCallback) -> i32>,
    pub create: u32,        // CreateFn<T>,
}
const _: [(); std::mem::size_of::<PluginDescriptorInfo32>()] = [(); 0x34];
unsafe impl Pod for PluginDescriptorInfo32 {}

// The padding inside the struct is only really required for targets
// which ignore the aligment property.
// Most notable for i686 cross-compilation the padding is required.
#[repr(C, align(8))]
struct PluginDescriptorInfo64 {
    pub plugin_version: i32,
    pub accept_input: u32,  // bool
    pub input_layout: u64,  // &'static TypeLayout
    pub output_layout: u64, // &'static TypeLayout,
    pub name: u64,          // CSliceRef<'static, u8>,
    pub name_length: u32,
    _pad0: u32,
    pub version: u64, // CSliceRef<'static, u8>,
    pub version_length: u32,
    _pad1: u32,
    pub description: u64, //CSliceRef<'static, u8>,
    pub description_length: u32,
    _pad2: u32,
    pub help_callback: u64, // Option<extern "C" fn(callback: HelpCallback) -> ()>,
    pub target_list_callback: u64, // Option<extern "C" fn(callback: TargetCallback) -> i32>,
    pub create: u64,        // CreateFn<T>,
}
const _: [(); std::mem::size_of::<PluginDescriptorInfo64>()] = [(); 0x60];
unsafe impl Pod for PluginDescriptorInfo64 {}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum PluginKind {
    Connector,
    Os,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum PluginFileType {
    Pe,
    Elf,
    Mach,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum PluginArchitecture {
    Unknown(u32),
    X86,
    X86_64,
    Arm,
    Arm64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PluginDescriptorInfo {
    pub plugin_kind: PluginKind,
    pub export_name: String,
    pub file_type: PluginFileType,
    pub architecture: PluginArchitecture,
    pub plugin_version: i32,
    pub name: String,
    pub version: String,
    pub description: String,
}

/// Peaks into the first 4 bytes of the header and matches it against a known set of binary magic constants.
pub fn is_binary(bytes: &[u8]) -> Result<()> {
    let view = DataView::from(bytes);
    let elfmag = u32::from_le_bytes(*ELFMAG);
    match view.read::<u32>(0) {
        tag if (tag as u16) == DOS_MAGIC => Ok(()),
        MH_MAGIC | MH_CIGAM | MH_MAGIC_64 | MH_CIGAM_64 => Ok(()),
        mag if mag == elfmag => Ok(()),
        tag => Err(Error(ErrorOrigin::Inventory, ErrorKind::NotSupported)
            .log_error(format!("unknown binary format (tag={:#}", tag))),
    }
}

/// Parses and returns all descriptors found in the binary.
/// This function tries to guess the binary type.
pub fn parse_descriptors(bytes: &[u8]) -> Result<Vec<PluginDescriptorInfo>> {
    let object = Object::parse(bytes).map_err(|err| {
        Error(ErrorOrigin::Inventory, ErrorKind::InvalidExeFile)
            .log_error(format!("unable to parse binary object: {}", err))
    })?;
    match object {
        Object::PE(pe) => pe_parse_descriptors(bytes, &pe),
        Object::Mach(mach) => mach_parse_descriptors(bytes, &mach),
        Object::Elf(elf) => elf_parse_descriptors(bytes, &elf),
        _ => Err(Error(ErrorOrigin::Inventory, ErrorKind::NotSupported)
            .log_error("unknown binary format")),
    }
}

/// Returns the plugin kind based of the export prefix
fn plugin_kind(export_name: &str) -> Result<PluginKind> {
    // stripping initial _ is required for MACH builds
    let export_name = export_name.strip_prefix('_').unwrap_or(export_name);

    // match by export prefix
    if export_name.starts_with(MEMFLOW_EXPORT_PREFIX_CONNECTOR) {
        Ok(PluginKind::Connector)
    } else if export_name.starts_with(MEMFLOW_EXPORT_PREFIX_OS) {
        Ok(PluginKind::Os)
    } else {
        Err(Error(ErrorOrigin::Inventory, ErrorKind::PluginNotFound))
    }
}

/// Parses the descriptors in a PE binary.
/// This function currently supports x86 and x86_64 binaries.
fn pe_parse_descriptors(bytes: &[u8], pe: &PE) -> Result<Vec<PluginDescriptorInfo>> {
    let mut ret = vec![];

    for export in pe.exports.iter() {
        if let Some(export_name) = export.name {
            if let Ok(plugin_kind) = plugin_kind(export_name) {
                if let Some(offset) = export.offset {
                    let data_view = DataView::from(bytes);

                    if pe.is_64 {
                        let raw_desc = data_view.read::<PluginDescriptorInfo64>(offset);
                        #[rustfmt::skip]
                        ret.push(PluginDescriptorInfo {
                            plugin_kind,
                            export_name: export_name.to_string(),
                            file_type: PluginFileType::Pe,
                            architecture: pe_architecture(pe),
                            plugin_version: raw_desc.plugin_version,
                            name: read_string(bytes, pe_va_to_offset(pe, raw_desc.name), raw_desc.name_length as usize)?,
                            version: read_string(bytes, pe_va_to_offset(pe, raw_desc.version), raw_desc.version_length as usize)?,
                            description: read_string(bytes, pe_va_to_offset(pe, raw_desc.description), raw_desc.description_length as usize)?,
                        });
                    } else {
                        let raw_desc = data_view.read::<PluginDescriptorInfo32>(offset);
                        #[rustfmt::skip]
                        ret.push(PluginDescriptorInfo {
                            plugin_kind,
                            export_name: export_name.to_string(),
                            file_type: PluginFileType::Pe,
                            architecture: pe_architecture(pe),
                            plugin_version: raw_desc.plugin_version,
                            name: read_string(bytes, pe_va_to_offset(pe, raw_desc.name as u64), raw_desc.name_length as usize)?,
                            version: read_string(bytes, pe_va_to_offset(pe, raw_desc.version as u64), raw_desc.version_length as usize)?,
                            description: read_string(bytes, pe_va_to_offset(pe, raw_desc.description as u64), raw_desc.description_length as usize)?,
                        });
                    }
                }
            }
        }
    }

    Ok(ret)
}

fn pe_architecture(pe: &PE) -> PluginArchitecture {
    // https://learn.microsoft.com/en-us/windows/win32/debug/pe-format#machine-types
    match pe.header.coff_header.machine {
        0x14c => PluginArchitecture::X86,
        0x8664 => PluginArchitecture::X86_64,
        0x1c0 | 0x1c4 => PluginArchitecture::Arm,
        0xAA64 => PluginArchitecture::Arm64,
        _ => PluginArchitecture::Unknown(pe.header.coff_header.machine as u32),
    }
}

fn pe_va_to_offset(pe: &PE, va: u64) -> usize {
    let offset_va = va as usize - pe.image_base;
    let file_alignment = pe
        .header
        .optional_header
        .map(|h| h.windows_fields.file_alignment)
        .unwrap_or(512);
    pe::utils::find_offset(
        offset_va,
        &pe.sections,
        file_alignment,
        &ParseOptions::default(),
    )
    .unwrap_or(0)
}

fn read_string(bytes: &[u8], offset: usize, len: usize) -> Result<String> {
    if offset == 0 {
        if len <= 1 {
            // allow empty strings (e.g. for the description field)
            return Ok(String::new());
        } else {
            // we expect a string but found nothing
            return Err(Error(ErrorOrigin::Inventory, ErrorKind::NotFound)
                .log_error("unable to read referenced string in binary"));
        }
    }

    if offset + len > bytes.len() {
        return Err(Error(ErrorOrigin::Inventory, ErrorKind::OutOfBounds)
            .log_error("referenced string is outside of the file"));
    }
    let mut buffer = vec![0u8; len];
    buffer.copy_from_slice(&bytes[offset..offset + len]);

    let result = std::str::from_utf8(&buffer[..]).map_err(|err| {
        Error(ErrorOrigin::Inventory, ErrorKind::Encoding).log_error(format!(
            "unable to read referenced string in binary: {}",
            err
        ))
    })?;

    Ok(result.to_owned())
}

fn mach_parse_descriptors(bytes: &[u8], mach: &Mach) -> Result<Vec<PluginDescriptorInfo>> {
    let mut ret = vec![];

    match mach {
        Mach::Binary(macho) => {
            let mut descriptors = macho_parse_descriptors(bytes, macho)?;
            ret.append(&mut descriptors);
        }
        Mach::Fat(multiarch) => {
            let arches = multiarch.arches().map_err(|err| {
                Error(ErrorOrigin::Inventory, ErrorKind::InvalidExeFile)
                    .log_error(format!("unable to extract arches from mach: {}", err))
            })?;
            for (index, fatarch) in arches.iter().enumerate() {
                if let Ok(arch) = multiarch.get(index) {
                    match arch {
                        SingleArch::MachO(macho) => {
                            let offset = fatarch.offset as usize;
                            let len = fatarch.size as usize;
                            let mut descriptors =
                                macho_parse_descriptors(&bytes[offset..offset + len], &macho)?;
                            ret.append(&mut descriptors);
                        }
                        SingleArch::Archive(_) => {
                            return Err(Error(ErrorOrigin::Inventory, ErrorKind::NotSupported)
                                .log_error("mach archives are not supported yet"));
                        }
                    }
                }
            }
        }
    }

    Ok(ret)
}

fn macho_parse_descriptors(bytes: &[u8], macho: &MachO) -> Result<Vec<PluginDescriptorInfo>> {
    let mut ret = vec![];

    if !macho.little_endian {
        return Err(Error(ErrorOrigin::Inventory, ErrorKind::NotSupported)
            .log_error("big endian binaries are not supported yet"));
    }

    if let Ok(exports) = macho.exports() {
        for export in exports.iter() {
            if let Ok(plugin_kind) = plugin_kind(&export.name) {
                let offset = export.offset;

                let data_view = DataView::from(bytes);

                if macho.is_64 {
                    let raw_desc = data_view.read::<PluginDescriptorInfo64>(offset as usize);
                    #[rustfmt::skip]
                    ret.push(PluginDescriptorInfo{
                        plugin_kind,
                        export_name: export.name.to_string(),
                        file_type: PluginFileType::Mach,
                        architecture: macho_architecture(macho),
                        plugin_version: raw_desc.plugin_version,
                        name: read_string(bytes, macho_va_to_offset(raw_desc.name), raw_desc.name_length as usize)?,
                        version: read_string(bytes, macho_va_to_offset(raw_desc.version), raw_desc.version_length as usize)?,
                        description: read_string(bytes, macho_va_to_offset(raw_desc.description), raw_desc.description_length as usize)?,
                    });
                } else {
                    let raw_desc = data_view.read::<PluginDescriptorInfo32>(offset as usize);
                    #[rustfmt::skip]
                    ret.push(PluginDescriptorInfo{
                        plugin_kind,
                        export_name: export.name.to_string(),
                        file_type: PluginFileType::Mach,
                        architecture: macho_architecture(macho),
                        plugin_version: raw_desc.plugin_version,
                        name: read_string(bytes, macho_va_to_offset(raw_desc.name as u64), raw_desc.name_length as usize)?,
                        version: read_string(bytes, macho_va_to_offset(raw_desc.version as u64), raw_desc.version_length as usize)?,
                        description: read_string(bytes, macho_va_to_offset(raw_desc.description as u64), raw_desc.description_length as usize)?,
                    });
                }
            }
        }
    }

    Ok(ret)
}

fn macho_architecture(macho: &MachO) -> PluginArchitecture {
    // https://crystal-lang.org/api/0.24.0/Debug/MachO/CpuType.html
    match macho.header.cputype {
        7 => PluginArchitecture::X86,
        16777223 => PluginArchitecture::X86_64,
        12 => PluginArchitecture::Arm,
        16777228 => PluginArchitecture::Arm64,
        _ => PluginArchitecture::Unknown(macho.header.cputype),
    }
}

fn macho_va_to_offset(va: u64) -> usize {
    // TODO: why is this offset padded so high? is there a vm base somewhere?
    (va & 0xffff_ffff) as usize
}

/// Parses the descriptors in an ELF binary.
/// This function currently supports x86, x86_64, aarch64 and armv7.
fn elf_parse_descriptors(bytes: &[u8], elf: &Elf) -> Result<Vec<PluginDescriptorInfo>> {
    let mut ret = vec![];

    if !elf.little_endian {
        return Err(Error(ErrorOrigin::Inventory, ErrorKind::NotSupported)
            .log_error("big endian binaries are not supported yet"));
    }

    let iter = elf
        .dynsyms
        .iter()
        .filter(|s| !s.is_import())
        .filter_map(|s| elf.dynstrtab.get_at(s.st_name).map(|n| (s, n)));

    for (sym, export_name) in iter {
        if let Ok(plugin_kind) = plugin_kind(export_name) {
            if sym.st_shndx == SHN_XINDEX as usize {
                return Err(Error(ErrorOrigin::Inventory, ErrorKind::NotSupported)
                    .log_error("unsupported elf SHN_XINDEX header flag"));
            }

            // section
            let section = elf
                .program_headers
                .iter()
                .find(|h| h.vm_range().contains(&(sym.st_value as usize)))
                .ok_or_else(|| {
                    Error(ErrorOrigin::Inventory, ErrorKind::NotFound)
                        .log_error("could not find any section containing the plugin descriptor")
                })?;

            // compute proper file offset based on section
            let offset = section.p_offset + sym.st_value - section.p_vaddr;

            let data_view = DataView::from(bytes);

            if elf.is_64 {
                let mut raw_desc = data_view.read::<PluginDescriptorInfo64>(offset as usize);
                elf_apply_relocs::<u64, _>(
                    elf,
                    sym.st_value..sym.st_value + sym.st_size,
                    &mut raw_desc,
                )?;
                #[rustfmt::skip]
                ret.push(PluginDescriptorInfo{
                    plugin_kind,
                    export_name: export_name.to_string(),
                    file_type: PluginFileType::Elf,
                    architecture: elf_architecture(elf),
                    plugin_version: raw_desc.plugin_version,
                    name: read_string(bytes, raw_desc.name as usize, raw_desc.name_length as usize)?,
                    version: read_string(bytes, raw_desc.version as usize, raw_desc.version_length as usize)?,
                    description: read_string(bytes, raw_desc.description as usize, raw_desc.description_length as usize)?,
                });
            } else {
                let mut raw_desc = data_view.read::<PluginDescriptorInfo32>(offset as usize);
                elf_apply_relocs::<u32, _>(
                    elf,
                    sym.st_value..sym.st_value + sym.st_size,
                    &mut raw_desc,
                )?;
                #[rustfmt::skip]
                ret.push(PluginDescriptorInfo{
                    plugin_kind,
                    export_name: export_name.to_string(),
                    file_type: PluginFileType::Elf,
                    architecture: elf_architecture(elf),
                    plugin_version: raw_desc.plugin_version,
                    name: read_string(bytes, raw_desc.name as usize, raw_desc.name_length as usize)?,
                    version: read_string(bytes, raw_desc.version as usize, raw_desc.version_length as usize)?,
                    description: read_string(bytes, raw_desc.description as usize, raw_desc.description_length as usize)?,
                });
            }
        }
    }

    Ok(ret)
}

fn elf_architecture(elf: &Elf) -> PluginArchitecture {
    // https://refspecs.linuxfoundation.org/elf/gabi4+/ch4.eheader.html
    match elf.header.e_machine {
        3 => PluginArchitecture::X86,
        62 => PluginArchitecture::X86_64,
        40 => PluginArchitecture::Arm,
        183 => PluginArchitecture::Arm64,
        _ => PluginArchitecture::Unknown(elf.header.e_machine as u32),
    }
}

fn elf_apply_relocs<N, T>(elf: &Elf, va_range: Range<u64>, obj: &mut T) -> Result<()>
where
    N: Pod + Eq + Zero + NumCast + WrappingAdd + WrappingSub,
    T: Pod,
{
    for section_relocs in elf.shdr_relocs.iter() {
        for reloc in section_relocs.1.iter() {
            if reloc.r_offset >= va_range.start && reloc.r_offset < va_range.end {
                let field_offset = reloc.r_offset - va_range.start;

                let data_view = DataView::from_mut(obj);
                let value = data_view.read::<N>(field_offset as usize);

                // skip over entries that already contain the proper reference
                if value != N::zero() {
                    continue;
                }

                // https://chromium.googlesource.com/android_tools/+/8301b711a9ac7de56e9a9ff3dee0b2ebfc9a380f/ndk/sources/android/crazy_linker/src/crazy_linker_elf_relocations.cpp#36
                // TODO: generalize this check
                if reloc.r_type != 8 && reloc.r_type != 23 && reloc.r_type != 1027 {
                    return Err(Error(ErrorOrigin::Inventory, ErrorKind::NotSupported)
                        .log_error("only relative relocations are supported right now"));
                }

                // simulate a `wrapping_add_signed`
                let addend = reloc.r_addend.unwrap_or_default();
                let value = if addend > 0 {
                    value.wrapping_add(&(num_traits::cast(addend).unwrap()))
                } else {
                    value.wrapping_sub(&(num_traits::cast(-addend).unwrap()))
                };
                data_view.write::<N>(field_offset as usize, &value);
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const NAME: &str = "coredump";
    const VERSION: &str = "0.2.0";
    const DESCRIPTION: &str =
        "win32 coredump connector for the memflow physical memory introspection framework";

    #[test]
    fn test_pe_x86_64() {
        let file = include_bytes!("../../../assets/tests/memflow_coredump.x86_64.dll");

        assert_eq!(is_binary(&file[..]), Ok(()));

        assert_eq!(
            parse_descriptors(&file[..]).unwrap(),
            vec![PluginDescriptorInfo {
                plugin_kind: PluginKind::Connector,
                export_name: "MEMFLOW_CONNECTOR_COREDUMP".to_owned(),
                file_type: PluginFileType::Pe,
                architecture: PluginArchitecture::X86_64,
                plugin_version: 1,
                name: NAME.to_owned(),
                version: VERSION.to_owned(),
                description: DESCRIPTION.to_owned(),
            }]
        );
    }

    #[test]
    fn test_pe_x86() {
        let file = include_bytes!("../../../assets/tests/memflow_coredump.x86.dll");

        assert_eq!(is_binary(&file[..]), Ok(()));

        assert_eq!(
            parse_descriptors(&file[..]).unwrap(),
            vec![PluginDescriptorInfo {
                plugin_kind: PluginKind::Connector,
                export_name: "MEMFLOW_CONNECTOR_COREDUMP".to_owned(),
                file_type: PluginFileType::Pe,
                architecture: PluginArchitecture::X86,
                plugin_version: 1,
                name: NAME.to_owned(),
                version: VERSION.to_owned(),
                description: DESCRIPTION.to_owned(),
            }]
        );
    }

    #[test]
    fn test_elf_x86_64() {
        let file = include_bytes!("../../../assets/tests/libmemflow_coredump.x86_64.so");

        assert_eq!(is_binary(&file[..]), Ok(()));

        assert_eq!(
            parse_descriptors(&file[..]).unwrap(),
            vec![PluginDescriptorInfo {
                plugin_kind: PluginKind::Connector,
                export_name: "MEMFLOW_CONNECTOR_COREDUMP".to_owned(),
                file_type: PluginFileType::Elf,
                architecture: PluginArchitecture::X86_64,
                plugin_version: 1,
                name: NAME.to_owned(),
                version: VERSION.to_owned(),
                description: DESCRIPTION.to_owned(),
            }]
        );
    }

    #[test]
    fn test_elf_x86() {
        let file = include_bytes!("../../../assets/tests/libmemflow_coredump.x86.so");

        assert_eq!(is_binary(&file[..]), Ok(()));

        assert_eq!(
            parse_descriptors(&file[..]).unwrap(),
            vec![PluginDescriptorInfo {
                plugin_kind: PluginKind::Connector,
                export_name: "MEMFLOW_CONNECTOR_COREDUMP".to_owned(),
                file_type: PluginFileType::Elf,
                architecture: PluginArchitecture::X86,
                plugin_version: 1,
                name: NAME.to_owned(),
                version: VERSION.to_owned(),
                description: DESCRIPTION.to_owned(),
            }]
        );
    }

    #[test]
    fn test_elf_arm64() {
        let file = include_bytes!("../../../assets/tests/libmemflow_coredump.aarch64.so");

        assert_eq!(is_binary(&file[..]), Ok(()));

        assert_eq!(
            parse_descriptors(&file[..]).unwrap(),
            vec![PluginDescriptorInfo {
                plugin_kind: PluginKind::Connector,
                export_name: "MEMFLOW_CONNECTOR_COREDUMP".to_owned(),
                file_type: PluginFileType::Elf,
                architecture: PluginArchitecture::Arm64,
                plugin_version: 1,
                name: NAME.to_owned(),
                version: VERSION.to_owned(),
                description: DESCRIPTION.to_owned(),
            }]
        );
    }

    #[test]
    fn test_elf_arm() {
        let file = include_bytes!("../../../assets/tests/libmemflow_coredump.arm.so");

        assert_eq!(is_binary(&file[..]), Ok(()));

        assert_eq!(
            parse_descriptors(&file[..]).unwrap(),
            vec![PluginDescriptorInfo {
                plugin_kind: PluginKind::Connector,
                export_name: "MEMFLOW_CONNECTOR_COREDUMP".to_owned(),
                file_type: PluginFileType::Elf,
                architecture: PluginArchitecture::Arm,
                plugin_version: 1,
                name: NAME.to_owned(),
                version: VERSION.to_owned(),
                description: DESCRIPTION.to_owned(),
            }]
        );
    }

    #[test]
    fn test_mach_arm64() {
        let file = include_bytes!("../../../assets/tests/libmemflow_native.aarch64.dylib");

        assert_eq!(is_binary(&file[..]), Ok(()));

        assert_eq!(
            parse_descriptors(&file[..]).unwrap(),
            vec![PluginDescriptorInfo {
                plugin_kind: PluginKind::Os,
                export_name: "_MEMFLOW_OS_NATIVE".to_owned(),
                file_type: PluginFileType::Mach,
                architecture: PluginArchitecture::Arm64,
                plugin_version: 1,
                name: "native".to_owned(),
                version: "0.2.2".to_owned(),
                description: "System call based proxy-OS for memflow".to_owned(),
            }]
        );
    }
}
