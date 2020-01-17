use crate::error::{Error, Result};

use log::{info, trace, warn};

use flow_core::address::{Address, Length};
use flow_core::mem::*;

use crate::kernel::StartBlock;

use crate::kernel::ntos;

use pelite::{self, pe64::exports::Export, PeView};

pub fn find<T: PhysicalMemoryTrait + VirtualMemoryTrait>(
    mem: &mut T,
    start_block: &StartBlock,
    ntos: Address,
) -> Result<Address> {
    trace!("trying to find system eprocess");

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
pub fn find_exported<T: PhysicalMemoryTrait + VirtualMemoryTrait>(
    mem: &mut T,
    start_block: &StartBlock,
    ntos: Address,
) -> Result<Address> {
    let header_buf = ntos::try_fetch_pe_header(mem, start_block, ntos)?;
    let header = PeView::from_bytes(&header_buf)?;

    let sys_proc = match header.get_export_by_name("PsInitialSystemProcess")? {
        // PsActiveProcessHead
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
    let mut reader = VirtualMemory::with(mem, start_block.arch, start_block.dtb);
    let addr = reader.virt_read_addr(sys_proc)?;
    Ok(addr)
}

// scan in pdb

// scan in section
pub fn find_in_section<T: PhysicalMemoryTrait + VirtualMemoryTrait>(
    mem: &mut T,
    start_block: &StartBlock,
    ntos: Address,
) -> Result<Address> {
    // find section ALMOSTRO
    // scan for va of system process (dtb.va)
    // ... check if its 32 or 64bit

    let mut header_buf = vec![0; Length::from_mb(32).as_usize()];
    mem.virt_read_raw(start_block.arch, start_block.dtb, ntos, &mut header_buf)?;

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
