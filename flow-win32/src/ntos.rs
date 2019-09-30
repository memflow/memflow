use std::io::{Error, ErrorKind, Result};
use num::range_step;

use byteorder::{ByteOrder, LittleEndian};

use flow_core::machine::Machine;
use flow_core::cpu::Architecture;

use crate::dtb::DTB;

// VmmWinInit_FindNtosScan
pub fn find(machine: &mut Machine, dtb: DTB) -> Result<()> {
    // TODO: create system process around current dtb

    if dtb.arch == Architecture::X64 {
        if dtb.va != 0 {
            match find_x64_with_va(machine, dtb.va) {
                Ok(b) => return Ok(b),
                Err(e) => println!("Error: {}", e),
            }
        }
        
        match find_x64(machine) {
            Ok(b) => return Ok(b),
            Err(e) => println!("Error: {}", e),
        }
    } else {
        match find_x86(machine) {
            Ok(b) => return Ok(b),
            Err(e) => println!("Error: {}", e),
        }
    }

    Err(Error::new(ErrorKind::Other, "unable to find ntoskrnl.exe"))
}

// VmmWinInit_FindNtosScanHint64
fn find_x64_with_va(machine: &mut Machine, va: u64) -> Result<()> {
    // va was found previously

    // TODO: .rev()
    for base in range_step(va - 0x2000000, va & !0x1fffff, 0x200000) {
        // ...
        // VmmReadEx with sysProc ...
        let mem = machine.mem.read_physical_memory(base, 0x200000)?;

        /*
        mem
            .chunks_exact(0x1000)
            .filter(|c| LittleEndian::read_u32(&c) == 0x5a4d) // MZ
            .chunks_exact(8)
            .filter(|c| LittleEndian::read_u64(&c) == 0x45444F434C4F4F50) // POOLCODE
            .ok_or_else(|| Error::new(ErrorKind::Other, "unable to find x64 dtb in lowstub < 1M"))
            .and_then(|c| {
                // PE_GetModuleNameEx()
                // compare to ntoskrnl.exe
                // return current base + p
            })
            */
    }

    Ok(())
}

fn find_x64(machine: &mut Machine) -> Result<()> {
    Ok(())
}

fn find_x86(machine: &mut Machine) -> Result<()> {
    Ok(())
}