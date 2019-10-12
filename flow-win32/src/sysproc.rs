use log::{info, trace, warn};
use std::io::{Error, ErrorKind, Result};

use address::{Address, Length};
use mem::{PhysicalRead, VirtualRead};

use goblin::pe::options::ParseOptions;
use goblin::pe::PE;

use crate::dtb::DTB;

pub fn find<T: PhysicalRead + VirtualRead>(
    mem: &mut T,
    dtb: DTB,
    ntos: Address,
) -> Result<Address> {
    trace!("trying to find system eprocess");

    match find_exported(mem, dtb, ntos) {
        Ok(e) => return Ok(e),
        Err(e) => warn!("{}", e),
    }

    match find_in_section(mem, dtb, ntos) {
        Ok(e) => return Ok(e),
        Err(e) => warn!("{}", e),
    }

    Err(Error::new(
        ErrorKind::Other,
        "unable to find system eprocess",
    ))
}

// find from exported symbol
pub fn find_exported<T: PhysicalRead + VirtualRead>(
    mem: &mut T,
    dtb: DTB,
    ntos: Address,
) -> Result<Address> {
    let header_buf = mem.virt_read(dtb.arch, dtb.dtb, ntos, Length::from_mb(32))?;

    let mut pe_opts = ParseOptions::default();
    pe_opts.resolve_rva = false;

    let header = PE::parse_with_opts(&header_buf, &pe_opts).unwrap(); // TODO: error
    header
        .exports
        .iter()
        .filter(|e| e.name.unwrap_or_default() == "PsInitialSystemProcess")
        .inspect(|e| trace!("found eat entry: {:?}", e))
        .nth(0)
        .ok_or_else(|| {
            Error::new(
                ErrorKind::Other,
                "unable to find export PsInitialSystemProcess",
            )
        })
        .and_then(|e| Ok(ntos + Length::from(e.rva)))
}

// scan in section
pub fn find_in_section<T: PhysicalRead + VirtualRead>(
    mem: &mut T,
    dtb: DTB,
    ntos: Address,
) -> Result<Address> {
    // find section ALMOSTRO
    // scan for va of system process (dtb.va)
    // ... check if its 32 or 64bit

    let header_buf = mem.virt_read(dtb.arch, dtb.dtb, ntos, Length::from_mb(32))?;

    let mut pe_opts = ParseOptions::default();
    pe_opts.resolve_rva = false;

    let header = PE::parse_with_opts(&header_buf, &pe_opts).unwrap(); // TODO: error
    let sect = header
        .sections
        .iter()
        .filter(|s| String::from_utf8(s.name.to_vec()).unwrap_or_default() == "ALMOSTRO")
        .nth(0)
        .ok_or_else(|| Error::new(ErrorKind::Other, "unable to find section ALMOSTRO"))?;

    Err(Error::new(ErrorKind::Other, "not implemented yet"))
}
