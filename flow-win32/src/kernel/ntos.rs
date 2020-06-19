mod pehelper;
mod x64;
mod x86;

use crate::error::{Error, Result};
use crate::kernel::StartBlock;
use crate::pe::{self, MemoryPeViewContext};

use log::warn;

use flow_core::mem::VirtualMemory;
use flow_core::types::Address;

use pelite::{self, image::GUID, pe64::debug::CodeView};
use uuid::{self, Uuid};

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

    Err(Error::new("unable to find ntoskrnl.exe"))
}

#[derive(Debug, Clone)]
pub struct Win32GUID {
    pub file_name: String,
    pub guid: String,
}

// TODO: move to pe::...
pub fn find_guid<T: VirtualMemory + ?Sized>(
    virt_mem: &mut T,
    kernel_base: Address,
) -> Result<Win32GUID> {
    let ctx = MemoryPeViewContext::new(virt_mem, kernel_base).map_err(Error::new)?;
    let pe = pe::wrap_memory_pe_view(&ctx).map_err(Error::new)?;

    let debug = match pe.debug() {
        Ok(d) => d,
        Err(_) => return Err(Error::new("unable to read debug_data in pe header")),
    };

    let code_view = debug
        .iter()
        .map(|e| e.entry())
        .filter_map(std::result::Result::ok)
        .find(|&e| e.as_code_view().is_some())
        .ok_or_else(|| Error::new("unable to find codeview debug_data entry"))?
        .as_code_view()
        .unwrap(); // TODO: fix unwrap

    let signature = match code_view {
        CodeView::Cv70 { image, .. } => image.Signature,
        CodeView::Cv20 { .. } => {
            return Err(Error::new(
                "invalid code_view entry version 2 found, expected 7",
            ))
        }
    };

    Ok(Win32GUID {
        file_name: code_view.pdb_file_name().to_string(),
        guid: generate_guid(signature, code_view.age())?,
    })
}

// TODO: this function might be omitted in the future if this is merged to pelite internally
fn generate_guid(signature: GUID, age: u32) -> Result<String> {
    let uuid = Uuid::from_fields(
        signature.Data1,
        signature.Data2,
        signature.Data3,
        &signature.Data4,
    )
    .map_err(Error::new)?;

    Ok(format!(
        "{}{:X}",
        uuid.to_simple().to_string().to_uppercase(),
        age
    ))
}
