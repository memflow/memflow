use crate::error::{Error, Result};

use log::{info, trace, warn};

use flow_core::address::{Address, Length};
use flow_core::mem::*;

use goblin::pe::options::ParseOptions;
use goblin::pe::PE;

use crate::kernel::StartBlock;

pub fn find<T: PhysicalRead + VirtualRead>(
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
pub fn find_exported<T: PhysicalRead + VirtualRead>(
    mem: &mut T,
    start_block: &StartBlock,
    ntos: Address,
) -> Result<Address> {
    let reader = VirtualReader::with(mem, start_block.arch, start_block.dtb);
    let header_buf = reader.virt_read(ntos, Length::from_mb(32))?;

    let mut pe_opts = ParseOptions::default();
    pe_opts.resolve_rva = false;

    let header = PE::parse_with_opts(&header_buf, &pe_opts).unwrap(); // TODO: error
    let sys_proc = header
        .exports
        .iter()
        .filter(|e| e.name.unwrap_or_default() == "PsInitialSystemProcess") // PsActiveProcessHead
        .inspect(|e| info!("found eat entry: {:?}", e))
        .nth(0)
        .ok_or_else(|| Error::new("unable to find export PsInitialSystemProcess"))
        .and_then(|e| Ok(ntos + Length::from(e.rva)))?;

    // read value again
    // TODO: fallback for 32bit
    // TODO: wrap error properly
    let addr = reader.virt_read_addr(sys_proc)?;
    Ok(addr)
}

// scan in pdb

// scan in section
pub fn find_in_section<T: PhysicalRead + VirtualRead>(
    mem: &mut T,
    start_block: &StartBlock,
    ntos: Address,
) -> Result<Address> {
    // find section ALMOSTRO
    // scan for va of system process (dtb.va)
    // ... check if its 32 or 64bit

    let header_buf = mem.virt_read(start_block.arch, start_block.dtb, ntos, Length::from_mb(32))?;

    let mut pe_opts = ParseOptions::default();
    pe_opts.resolve_rva = false;

    let header = PE::parse_with_opts(&header_buf, &pe_opts).unwrap(); // TODO: error
    let _sect = header
        .sections
        .iter()
        .filter(|s| String::from_utf8(s.name.to_vec()).unwrap_or_default() == "ALMOSTRO")
        .nth(0)
        .ok_or_else(|| Error::new("unable to find section ALMOSTRO"))?;

    Err(Error::new("not implemented yet"))
}
