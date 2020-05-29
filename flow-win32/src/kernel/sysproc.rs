use crate::error::{Error, Result};

use byteorder::{ByteOrder, LittleEndian};
use log::{debug, info, warn};

use flow_core::mem::*;
use flow_core::types::{Address, Length};

use crate::kernel::StartBlock;

use crate::kernel::ntos;

use pelite::{self, pe64::exports::Export, PeView};

pub fn find<T: AccessPhysicalMemoryExt + AccessVirtualMemory + ?Sized>(
    mem: &mut T,
    start_block: &StartBlock,
    ntos: Address,
) -> Result<Address> {
    debug!("trying to find system eprocess");

    match find_exported(mem, start_block, ntos) {
        Ok(e) => return Ok(e),
        Err(e) => warn!("{}", e),
    }

    match find_in_section(mem, start_block, ntos) {
        Ok(e) => return Ok(e),
        Err(e) => warn!("{}", e),
    }

    Err(Error::new("unable to find system eprocess"))
}

// find from exported symbol
pub fn find_exported<T: AccessPhysicalMemoryExt + AccessVirtualMemory + ?Sized>(
    mem: &mut T,
    start_block: &StartBlock,
    ntos: Address,
) -> Result<Address> {
    let header_buf = ntos::pe::try_fetch_pe_header(mem, start_block, ntos)?;
    let header = PeView::from_bytes(&header_buf)?;

    // PsInitialSystemProcess -> PsActiveProcessHead
    let sys_proc = match header.get_export_by_name("PsInitialSystemProcess")? {
        Export::Symbol(s) => ntos + Length::from(*s),
        Export::Forward(_) => {
            return Err(Error::new(
                "PsInitialSystemProcess found but it was a forwarded export",
            ))
        }
    };

    info!("PsInitialSystemProcess found at 0x{:x}", sys_proc);

    // read value again
    // TODO: fallback for 32bit
    // TODO: wrap error properly
    let mut out = vec![0u8; start_block.arch.len_addr().as_usize()];
    mem.virt_read_raw_into(start_block.arch, start_block.dtb, sys_proc, &mut out)?;
    let address: Address = if start_block.arch.bits() == 64 {
        LittleEndian::read_u64(&out).into()
    } else if start_block.arch.bits() == 32 {
        LittleEndian::read_u32(&out).into()
    } else {
        return Err(Error::new(
            "invalid address size for this architecture. windows requires either 64 or 32 bits.",
        ));
    };
    Ok(address)
}

// scan in pdb

// scan in section
pub fn find_in_section<T: AccessPhysicalMemoryExt + AccessVirtualMemory + ?Sized>(
    mem: &mut T,
    start_block: &StartBlock,
    ntos: Address,
) -> Result<Address> {
    // find section ALMOSTRO
    // scan for va of system process (dtb.va)
    // ... check if its 32 or 64bit

    let mut header_buf = vec![0; Length::from_mb(32).as_usize()];
    mem.virt_read_raw_into(start_block.arch, start_block.dtb, ntos, &mut header_buf)?;

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

    Err(Error::new("not implemented yet"))
}
