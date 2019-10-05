use std::io::{Error, ErrorKind, Result};
use num::range_step;

use byteorder::{ByteOrder, LittleEndian};

use flow_core::arch::InstructionSet;
use flow_core::address::Address;
use flow_core::mem::{PhysicalRead, VirtualRead};

use crate::dtb::DTB;

// VmmWinInit_FindNtosScan
pub fn find<T: PhysicalRead + VirtualRead>(mem: &mut T, dtb: DTB) -> Result<()> {
    // TODO: create system process around current dtb

    if dtb.arch.instruction_set == InstructionSet::X64 {
        if !dtb.va.is_null() {
            match find_x64_with_va(mem, dtb.va) {
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
fn find_x64_with_va<T: PhysicalRead + VirtualRead>(mem: &mut T, va: Address) -> Result<()> {
    // va was found previously

    // TODO: .rev()
//    for base in range_step(va - 0x2000000, va & !0x1fffff, 0x200000) {
        // ...
        // VmmReadEx with sysProc ...
  //      let mem = machine.mem.read_memory(base, 0x200000)?;

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
   // }

    Ok(())
}

fn find_x64<T: PhysicalRead + VirtualRead>(mem: &mut T) -> Result<()> {
    Ok(())
}

fn find_x86<T: PhysicalRead + VirtualRead>(mem: &mut T) -> Result<()> {
    Ok(())
}