use crate::error::{Error, Result};

use log::{info, trace, warn};

use address::{Address, Length};
use mem::{PhysicalRead, VirtualRead};

use goblin::pe::options::ParseOptions;
use goblin::pe::PE;

use crate::kernel::KernelStubInfo;

pub fn find<T: PhysicalRead + VirtualRead>(
    mem: &mut T,
    stub_info: &KernelStubInfo,
    ntos: Address,
) -> Result<Address> {
    trace!("trying to find system eprocess");

    match find_exported(mem, stub_info, ntos) {
        Ok(e) => return Ok(e),
        Err(e) => warn!("{}", e),
    }

    match find_in_section(mem, stub_info, ntos) {
        Ok(e) => return Ok(e),
        Err(e) => warn!("{}", e),
    }

    Err(Error::new("unable to find system eprocess"))
}

// find from exported symbol
pub fn find_exported<T: PhysicalRead + VirtualRead>(
    mem: &mut T,
    stub_info: &KernelStubInfo,
    ntos: Address,
) -> Result<Address> {
    let header_buf = mem.virt_read(stub_info.arch, stub_info.dtb, ntos, Length::from_mb(32))?;

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

    //mem.virt_read_addr(stub_info.arch, stub_info.dtb, addr: Address).unwrap();
    Ok(Address::null())
}

// scan in pdb

// scan in section
pub fn find_in_section<T: PhysicalRead + VirtualRead>(
    mem: &mut T,
    stub_info: &KernelStubInfo,
    ntos: Address,
) -> Result<Address> {
    // find section ALMOSTRO
    // scan for va of system process (dtb.va)
    // ... check if its 32 or 64bit

    let header_buf = mem.virt_read(stub_info.arch, stub_info.dtb, ntos, Length::from_mb(32))?;

    let mut pe_opts = ParseOptions::default();
    pe_opts.resolve_rva = false;

    let header = PE::parse_with_opts(&header_buf, &pe_opts).unwrap(); // TODO: error
    let sect = header
        .sections
        .iter()
        .filter(|s| String::from_utf8(s.name.to_vec()).unwrap_or_default() == "ALMOSTRO")
        .nth(0)
        .ok_or_else(|| Error::new("unable to find section ALMOSTRO"))?;

    Err(Error::new("not implemented yet"))
}
