use std::prelude::v1::*;

use super::ntos::pehelper;
use super::StartBlock;
use crate::error::{Error, Result};

use std::convert::TryInto;

use log::{debug, info, warn};

use memflow::mem::VirtualMemory;
use memflow::types::{size, Address};

use pelite::{self, pe64::exports::Export, PeView};

pub fn find<T: VirtualMemory>(
    virt_mem: &mut T,
    start_block: &StartBlock,
    ntos: Address,
) -> Result<Address> {
    debug!("trying to find system eprocess");

    match find_exported(virt_mem, start_block, ntos) {
        Ok(e) => return Ok(e),
        Err(e) => warn!("{}", e),
    }

    match find_in_section(virt_mem, start_block, ntos) {
        Ok(e) => return Ok(e),
        Err(e) => warn!("{}", e),
    }

    Err(Error::Initialization("unable to find system eprocess"))
}

// find from exported symbol
pub fn find_exported<T: VirtualMemory>(
    virt_mem: &mut T,
    start_block: &StartBlock,
    kernel_base: Address,
) -> Result<Address> {
    // PsInitialSystemProcess -> PsActiveProcessHead
    let image = pehelper::try_get_pe_image(virt_mem, kernel_base)?;
    let pe = PeView::from_bytes(&image).map_err(Error::PE)?;

    let sys_proc = match pe
        .get_export_by_name("PsInitialSystemProcess")
        .map_err(Error::PE)?
    {
        Export::Symbol(s) => kernel_base + *s as usize,
        Export::Forward(_) => {
            return Err(Error::Other(
                "PsInitialSystemProcess found but it was a forwarded export",
            ))
        }
    };
    info!("PsInitialSystemProcess found at 0x{:x}", sys_proc);

    // read containing value
    let mut buf = vec![0u8; start_block.arch.size_addr()];
    let sys_proc_addr: Address = match start_block.arch.bits() {
        64 => {
            // TODO: replace by virt_read_into with ByteSwap
            virt_mem.virt_read_raw_into(sys_proc, &mut buf)?;
            u64::from_le_bytes(buf[0..8].try_into().unwrap()).into()
        }
        32 => {
            // TODO: replace by virt_read_into with ByteSwap
            virt_mem.virt_read_raw_into(sys_proc, &mut buf)?;
            u32::from_le_bytes(buf[0..4].try_into().unwrap()).into()
        }
        _ => return Err(Error::InvalidArchitecture),
    };
    Ok(sys_proc_addr)
}

// scan in pdb

// scan in section
pub fn find_in_section<T: VirtualMemory>(
    virt_mem: &mut T,
    _start_block: &StartBlock,
    ntos: Address,
) -> Result<Address> {
    // find section ALMOSTRO
    // scan for va of system process (dtb.va)
    // ... check if its 32 or 64bit

    let mut header_buf = vec![0; size::mb(32)];
    virt_mem.virt_read_raw_into(ntos, &mut header_buf)?;

    /*
    let mut pe_opts = ParseOptions::default();
    pe_opts.resolve_rva = false;

    let header = PE::parse_with_opts(&header_buf, &pe_opts).unwrap(); // TODO: error
    let _sect = header
        .sections
        .iter()
        .filter(|s| String::from_utf8(s.name.to_vec()).unwrap_or_default() == "ALMOSTRO")
        .nth(0)
        .ok_or_else(|| Error::new("unable to find section ALMOSTRO"))?;
    */

    Err(Error::Other(
        "sysproc::find_in_section(): not implemented yet",
    ))
}
