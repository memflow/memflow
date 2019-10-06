use std::io::{Error, ErrorKind, Result};
use num::range_step;

use byteorder::{ByteOrder, LittleEndian};

use arch::{Architecture, InstructionSet};
use address::{Address, Length};
use mem::{PhysicalRead, VirtualRead};

use crate::dtb::DTB;

// VmmWinInit_FindNtosScan
pub fn find<T: PhysicalRead + VirtualRead>(mem: &mut T, dtb: DTB) -> Result<Address> {
    // TODO: create system process around current dtb

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

// VmmWinInit_FindNtosScanHint64
fn find_x64_with_va<T: PhysicalRead + VirtualRead>(mem: &mut T, dtb: &DTB) -> Result<Address> {
    println!("find_x64_with_va(): trying to find ntoskrnl.exe with va hint {:x}", dtb.va.as_u64());

    // va was found previously
    // TODO: use address structure for this as well!
    let mut va_base = dtb.va.as_u64() & !0x1fffff;
    while va_base + Length::from_mb(32).as_u64() > dtb.va.as_u64() {
        println!("trying to read {:x}", va_base);
        let buf = mem.virt_read(dtb.arch, dtb.dtb, Address::from(va_base), Length::from_mb(2))?;
        if buf.is_empty() {
            // TODO: print address as well
            return Err(Error::new(ErrorKind::Other, "Unable to read memory when scanning for ntoskrnl.exe"))
        }
println!("found buf with len {}", buf.len());

        let res = buf
            .chunks_exact(0x1000)
            .enumerate()
            .filter(|(_, c)| LittleEndian::read_u32(&c) == 0x5a4d) // MZ
            .inspect(|(_, _)| println!("found MZ header"))
            .flat_map(|(i, c)| c.chunks_exact(8).map(move |c| (i, c)))
            .filter(|(_, c)| LittleEndian::read_u64(&c) == 0x45444F434C4F4F50) // POOLCODE
            .filter(|(_, _)| {
                // check for module name
                true
            })
            .nth(0)
            .ok_or_else(|| Error::new(ErrorKind::Other, "unable to find ntoskrnl.exe with va hint"))
            .and_then(|(i, _)| {
                // PE_GetModuleNameEx()
                // compare to ntoskrnl.exe
                // return current base + p
                // ...
                Ok(va_base + i as u64 * 0x1000)
            });

        match res {
            Ok(b) => return Ok(Address::from(b)),
            Err(_) => (),
        }

        va_base -= Length::from_mb(2).as_u64();
    }

    Err(Error::new(ErrorKind::Other, "unable to find ntoskrnl.exe with va hint"))
}

fn find_x64<T: PhysicalRead + VirtualRead>(mem: &mut T) -> Result<Address> {
    Err(Error::new(ErrorKind::Other, "find_x64(): not implemented yet"))
}

fn find_x86<T: PhysicalRead + VirtualRead>(mem: &mut T) -> Result<Address> {
    Err(Error::new(ErrorKind::Other, "find_x86(): not implemented yet"))
}