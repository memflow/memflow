pub(crate) mod pehelper;

mod x64;
mod x86;

use super::{StartBlock, Win32GUID, Win32Version};
use crate::error::{Error, PartialResultExt, Result};

use std::convert::TryInto;
use std::prelude::v1::*;

use log::{info, warn};

use memflow::mem::VirtualMemory;
use memflow::types::Address;

use pelite::{self, pe64::debug::CodeView, pe64::exports::Export, PeView};

pub fn find<T: VirtualMemory>(
    virt_mem: &mut T,
    start_block: &StartBlock,
) -> Result<(Address, usize)> {
    if start_block.arch.bits() == 64 {
        if !start_block.kernel_hint.is_null() {
            match x64::find_with_va_hint(virt_mem, start_block) {
                Ok(b) => return Ok(b),
                Err(e) => warn!("x64::find_with_va_hint() error: {}", e),
            }
        }

        match x64::find(virt_mem, start_block) {
            Ok(b) => return Ok(b),
            Err(e) => warn!("x64::find() error: {}", e),
        }
    } else if start_block.arch.bits() == 32 {
        match x86::find(virt_mem, start_block) {
            Ok(b) => return Ok(b),
            Err(e) => warn!("x86::find() error: {}", e),
        }
    }

    Err(Error::Initialization("unable to find ntoskrnl.exe"))
}

// TODO: move to pe::...
pub fn find_guid<T: VirtualMemory>(virt_mem: &mut T, kernel_base: Address) -> Result<Win32GUID> {
    let image = pehelper::try_get_pe_image(virt_mem, kernel_base)?;
    let pe = PeView::from_bytes(&image).map_err(Error::PE)?;

    let debug = match pe.debug() {
        Ok(d) => d,
        Err(_) => {
            return Err(Error::Initialization(
                "unable to read debug_data in pe header",
            ))
        }
    };

    let code_view = debug
        .iter()
        .map(|e| e.entry())
        .filter_map(std::result::Result::ok)
        .find(|&e| e.as_code_view().is_some())
        .ok_or_else(|| Error::Initialization("unable to find codeview debug_data entry"))?
        .as_code_view()
        .ok_or_else(|| Error::PE(pelite::Error::Unmapped))?;

    let signature = match code_view {
        CodeView::Cv70 { image, .. } => image.Signature,
        CodeView::Cv20 { .. } => {
            return Err(Error::Initialization(
                "invalid code_view entry version 2 found, expected 7",
            ))
        }
    };

    let file_name = code_view.pdb_file_name().to_str()?;
    let guid = format!("{:X}{:X}", signature, code_view.age());
    Ok(Win32GUID::new(file_name, &guid))
}

fn get_export(pe: &PeView, name: &str) -> Result<usize> {
    info!("trying to find {} export", name);
    let export = match pe.get_export_by_name(name).map_err(Error::PE)? {
        Export::Symbol(s) => *s as usize,
        Export::Forward(_) => {
            return Err(Error::Other("Export found but it was a forwarded export"))
        }
    };
    info!("{} found at 0x{:x}", name, export);
    Ok(export)
}

pub fn find_winver<T: VirtualMemory>(
    virt_mem: &mut T,
    kernel_base: Address,
) -> Result<Win32Version> {
    let image = pehelper::try_get_pe_image(virt_mem, kernel_base)?;
    let pe = PeView::from_bytes(&image).map_err(Error::PE)?;

    // NtBuildNumber
    let nt_build_number_ref = get_export(&pe, "NtBuildNumber")?;
    let rtl_get_version_ref = get_export(&pe, "RtlGetVersion");

    let nt_build_number: u32 = virt_mem.virt_read(kernel_base + nt_build_number_ref)?;
    info!("nt_build_number: {}", nt_build_number);
    if nt_build_number == 0 {
        return Err(Error::Initialization("unable to fetch nt build number"));
    }

    // TODO: these reads should be optional
    // try to find major/minor version
    // read from KUSER_SHARED_DATA. these fields exist since nt 4.0 so they have to exist in case NtBuildNumber exists.
    let mut nt_major_version: u32 = virt_mem
        .virt_read((0x7ffe0000 + 0x026C).into())
        .data_part()?;
    let mut nt_minor_version: u32 = virt_mem
        .virt_read((0x7ffe0000 + 0x0270).into())
        .data_part()?;

    // fallback on x64: try to parse RtlGetVersion assembly
    if nt_major_version == 0 && rtl_get_version_ref.is_ok() {
        let mut buf = [0u8; 0x100];
        virt_mem
            .virt_read_into(kernel_base + rtl_get_version_ref.unwrap(), &mut buf)
            .data_part()?;

        nt_major_version = 0;
        nt_minor_version = 0;

        for i in 0..0xf0 {
            if nt_major_version == 0
                && nt_minor_version == 0
                && u32::from_le_bytes(buf[i..i + 4].try_into().unwrap()) == 0x441c748
            {
                nt_major_version =
                    u16::from_le_bytes(buf[i + 4..i + 4 + 2].try_into().unwrap()) as u32;
                nt_minor_version = (buf[i + 5] & 0xF) as u32;
            }

            if nt_major_version == 0
                && u32::from_le_bytes(buf[i..i + 4].try_into().unwrap()) & 0xFFFFF == 0x441c7
            {
                nt_major_version = buf[i + 3] as u32;
            }

            if nt_minor_version == 0
                && u32::from_le_bytes(buf[i..i + 4].try_into().unwrap()) & 0xFFFFF == 0x841c7
            {
                nt_major_version = buf[i + 3] as u32;
            }
        }
    }

    // construct Win32BuildNumber object (major and minor version might be null but build number should be set)
    let version = Win32Version::new(nt_major_version, nt_minor_version, nt_build_number);
    info!("kernel version: {}", version);

    Ok(version)
}
