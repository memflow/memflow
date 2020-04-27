pub mod pe;

mod x64;
mod x86;

use crate::error::{Error, Result};
use crate::kernel::StartBlock;

use log::warn;
use pelite::{self, image::GUID, pe64::debug::CodeView, PeView};
use uuid::{self, Uuid};

use flow_core::address::{Address, Length};
use flow_core::mem::AccessVirtualMemory;

pub fn find<T: AccessVirtualMemory>(
    mem: &mut T,
    start_block: &StartBlock,
) -> Result<(Address, Length)> {
    if start_block.arch.bits() == 64 {
        if !start_block.va.is_null() {
            match x64::find_with_va(mem, start_block) {
                Ok(b) => return Ok(b),
                Err(e) => warn!("{}", e),
            }
        }

        match x64::find(mem) {
            Ok(b) => return Ok(b),
            Err(e) => warn!("{}", e),
        }
    } else if start_block.arch.bits() == 32 {
        match x86::find(mem) {
            Ok(b) => return Ok(b),
            Err(e) => println!("Error: {}", e),
        }
    }

    Err(Error::new("unable to find ntoskrnl.exe"))
}

#[derive(Debug, Clone)]
pub struct Win32GUID {
    pub file_name: String,
    pub guid: String,
}

pub fn find_guid<T: AccessVirtualMemory>(
    mem: &mut T,
    start_block: &StartBlock,
    kernel_base: Address,
    kernel_size: Length,
) -> Result<Win32GUID> {
    let mut pe_buf = vec![0; kernel_size.as_usize()];
    mem.virt_read_raw_into(start_block.arch, start_block.dtb, kernel_base, &mut pe_buf)?;

    let pe = PeView::from_bytes(&pe_buf)?;

    let debug = match pe.debug() {
        Ok(d) => d,
        Err(_) => return Err(Error::new("unable to read debug_data in pe header")),
    };

    let code_view = debug
        .iter()
        .map(|e| e.entry())
        .filter_map(std::result::Result::ok)
        .filter(|&e| e.as_code_view().is_some())
        .nth(0)
        .ok_or_else(|| Error::new("unable to find codeview debug_data entry"))?
        .as_code_view()
        .unwrap();

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
    )?;

    Ok(format!(
        "{}{:X}",
        uuid.to_simple().to_string().to_uppercase(),
        age
    ))
}
