use log::{info, trace, debug};

use std::io::{Error, ErrorKind, Result};

use byteorder::{ByteOrder, LittleEndian};

use arch::{InstructionSet};
use address::{Address, Length};
use mem::{PhysicalRead, VirtualRead};

use goblin::pe::PE;
use goblin::pe::options::ParseOptions;

use crate::dtb::DTB;

// TODO: -> Result<WinProcess>
pub fn find<T: PhysicalRead + VirtualRead>(mem: &mut T, dtb: DTB) -> Result<Address> {
    if dtb.arch.instruction_set == InstructionSet::X64 {
        if !dtb.va.is_null() {
            match find_x64_with_va(mem, &dtb) {
                Ok(b) => return Ok(b),
                Err(e) => println!("Error: {}", e),
            }
        }
        
        match find_x64(mem) {
            Ok(b) => return Ok(b),
            Err(e) => println!("Error: {}", e),
        }
    } else {
        match find_x86(mem) {
            Ok(b) => return Ok(b),
            Err(e) => println!("Error: {}", e),
        }
    }

    Err(Error::new(ErrorKind::Other, "unable to find ntoskrnl.exe"))
}

fn find_x64_with_va<T: PhysicalRead + VirtualRead>(mem: &mut T, dtb: &DTB) -> Result<Address> {
    trace!("find_x64_with_va: trying to find ntoskrnl.exe with va hint at {:x}", dtb.va.as_u64());

    // va was found previously
    let mut va_base = dtb.va.as_u64() & !0x1fffff;
    while va_base + Length::from_mb(32).as_u64() > dtb.va.as_u64() {
        trace!("find_x64_with_va: probing at {:x}", va_base);

        let buf = mem.virt_read(dtb.arch, dtb.dtb, Address::from(va_base), Length::from_mb(2))?;
        if buf.is_empty() {
            // TODO: print address as well
            return Err(Error::new(ErrorKind::Other, "Unable to read memory when scanning for ntoskrnl.exe"))
        }

        let res = buf
            .chunks_exact(0x1000)
            .enumerate()
            .filter(|(_, c)| LittleEndian::read_u16(&c) == 0x5a4d) // MZ
            .inspect(|(i, _)| trace!("find_x64_with_va: found potential MZ flag at offset {:x}", i * 0x1000))
            .flat_map(|(i, c)| c.chunks_exact(8).map(move |c| (i, c)))
            .filter(|(_, c)| LittleEndian::read_u64(&c) == 0x45444F434C4F4F50) // POOLCODE
            .inspect(|(i, _)| trace!("find_x64_with_va: found potential POOLCODE flag at offset {:x}", i * 0x1000))
            .filter(|(i, c)| {
                // try to probe pe header
                let probe_addr = Address::from(va_base + (*i as u64) * 0x1000);
                let probe_buf = mem.virt_read(dtb.arch, dtb.dtb, probe_addr, Length::from_mb(32)).unwrap();

                let mut pe_opts = ParseOptions::default();
                pe_opts.resolve_rva = false;

                let pe = match PE::parse_with_opts(&probe_buf, &pe_opts) {
                    Ok(pe) => {
                        trace!("find_x64_with_va: found pe header:\n{:?}", pe);
                        pe
                    },
                    Err(e) => {
                        trace!("find_x64_with_va: potential pe header at offset {:x} could not be probed: {:?}", i * 0x1000, e);
                        return false;
                    }
                };

                info!("find_x64_with_va: found pe header for {}", pe.name.unwrap_or_default());
                return pe.name.unwrap_or_default() == "ntoskrnl.exe"
            })
            .nth(0)
            .ok_or_else(|| {
                Error::new(ErrorKind::Other, "find_x64_with_va: unable to locate ntoskrnl.exe via va hint")
            })
            .and_then(|(i, _)| {
                Ok(va_base + i as u64 * 0x1000)
            });
        match res {
            Ok(a) => return Ok(Address::from(a)),
            Err(e) => debug!("{:?}", e),
        }

        va_base -= Length::from_mb(2).as_u64();
    }

    Err(Error::new(ErrorKind::Other, "find_x64_with_va: unable to locate ntoskrnl.exe via va hint"))
}

fn find_x64<T: PhysicalRead + VirtualRead>(mem: &mut T) -> Result<Address> {
    Err(Error::new(ErrorKind::Other, "find_x64(): not implemented yet"))
}

fn find_x86<T: PhysicalRead + VirtualRead>(mem: &mut T) -> Result<Address> {
    Err(Error::new(ErrorKind::Other, "find_x86(): not implemented yet"))
}
