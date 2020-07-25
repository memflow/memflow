mod pehelper;
mod x64;
mod x86;

use std::prelude::v1::*;

use crate::error::{Error, Result};
use crate::kernel::StartBlock;
use crate::offsets::Win32GUID;
use crate::pe::{self, MemoryPeViewContext};

use log::warn;

use memflow_core::mem::VirtualMemory;
use memflow_core::types::Address;

use pelite::{self, pe64::debug::CodeView};

pub fn find<T: VirtualMemory + ?Sized>(
    virt_mem: &mut T,
    start_block: &StartBlock,
) -> Result<(Address, usize)> {
    if start_block.arch.bits() == 64 {
        if !start_block.kernel_hint.is_null() {
            match x64::find_with_va(virt_mem, start_block) {
                Ok(b) => return Ok(b),
                Err(e) => warn!("x64::find_with_va() error: {}", e),
            }
        }

        match x64::find(virt_mem) {
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
pub fn find_guid<T: VirtualMemory + ?Sized>(
    virt_mem: &mut T,
    kernel_base: Address,
) -> Result<Win32GUID> {
    let ctx = MemoryPeViewContext::new(virt_mem, kernel_base).map_err(Error::PE)?;
    let pe = pe::wrap_memory_pe_view(&ctx).map_err(Error::PE)?;

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
        .unwrap(); // TODO: fix unwrap

    let signature = match code_view {
        CodeView::Cv70 { image, .. } => image.Signature,
        CodeView::Cv20 { .. } => {
            return Err(Error::Initialization(
                "invalid code_view entry version 2 found, expected 7",
            ))
        }
    };

    Ok(Win32GUID {
        file_name: code_view.pdb_file_name().to_string(),
        guid: format!("{:X}{:X}", signature, code_view.age()),
    })
}

// TODO: move to pe::...
pub fn find_version<T: VirtualMemory + ?Sized>(
    virt_mem: &mut T,
    kernel_base: Address,
) -> Result<()> {
    let ctx = MemoryPeViewContext::new(virt_mem, kernel_base).map_err(Error::PE)?;
    let pe = pe::wrap_memory_pe_view(&ctx).map_err(Error::PE)?;

    //    println!("resources:\n{:?}", pe.resources()?);

    //   let verRes = pe.resources()?.find("VERSION").map_err(|_| Error::Other("unable to get winver"))?;
    //  println!("ver: {:?}", verRes);
    println!(
        "version: {:?}",
        pe.resources()?
            .version_info()
            .map_err(|_| Error::Other("unable to get winver"))?
    );

    /*
    let winver_bytes = pe
        .resources()?
        .root()?
        .get_dir("#VERSION".into()).map_err(|_| Error::Other("unable to get winver"))?
        .get_dir("#1".into()).map_err(|_| Error::Other("unable to get winver"))?
        .first().map_err(|_| Error::Other("unable to get winver"))?
        .data().ok_or_else(|| Error::Other("unable to get winver"))?
        .bytes()?;
    println!("winver_bytes: {:?}", winver_bytes);
    let winver = String::from_utf8(winver_bytes.to_vec());

    println!("winver: {:?}", winver);
    */

    Ok(())
}
