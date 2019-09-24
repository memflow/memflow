// TODO: custom errors
use std::io::{Error, ErrorKind, Result};

use byteorder::{ByteOrder, LittleEndian};

use flow_core::{machine::Machine};

pub struct DTB {
    pub va: u64,
    pub dtb: u64,
}

pub struct Windows {
    dtb: DTB
}

pub fn find(machine: &mut Machine) -> Result<DTB> {
    let low16 = machine.mem.read_physical_memory(0, 0x100000)?;

    // TODO: prints
    // println!("found x64 dtb in lowstub < 1M: va={:x} dtb={:x}", va, dtb);

    // find x64 dtb in low stub < 1M
    match find_x64_lowstub(&low16) {
        Ok(d) => return Ok(d),
        Err(e) => println!("Error: {}", e),
    }

    match find_x64(&low16) {
        Ok(d) => return Ok(d),
        Err(e) => println!("Error: {}", e),
    }

    match find_x64_pae(&low16) {
        Ok(d) => return Ok(d),
        Err(e) => println!("Error: {}", e),
    }

    match find_x86(&low16) {
        Ok(d) => return Ok(d),
        Err(e) => println!("Error: {}", e),
    }

    Err(Error::new(ErrorKind::Other, "unable to find dtb"))
}

pub fn find_x64_lowstub(stub: &Vec<u8>) -> Result<DTB> {
    stub
        .chunks_exact(0x1000)
        .skip(1)
        .filter(|c| (0xffffffffffff00ff & LittleEndian::read_u64(&c)) == 0x00000001000600E9) // start bytes
        .filter(|c| (0xfffff80000000003 & LittleEndian::read_u64(&c[0x70..])) == 0xfffff80000000000) // kernel entry
        .filter(|c| (0xffffff0000000fff & LittleEndian::read_u64(&c[0xA0..])) == 0) // pml4
        .nth(0)
        .ok_or(Error::new(ErrorKind::Other, "unable to find x64 dtb in lowstub < 1M"))
        .and_then(|c| {
            Ok(DTB{
                va: LittleEndian::read_u64(&c[0x70..]),
                dtb: LittleEndian::read_u64(&c[0xA0..]),
            })
        })
}

pub fn find_x64(mem: &Vec<u8>) -> Result<DTB> {
    Err(Error::new(ErrorKind::Other, "unable to find x64 dtb in lowstub < 16M"))
}

pub fn find_x64_pae(mem: &Vec<u8>) -> Result<DTB> {
    Err(Error::new(ErrorKind::Other, "unable to find x64_pae dtb in lowstub < 16M"))
}

pub fn find_x86(mem: &Vec<u8>) -> Result<DTB> {
    Err(Error::new(ErrorKind::Other, "unable to find x86 dtb in lowstub < 16M"))
}
