// TODO: custom errors
use std::io::{Error, ErrorKind, Result};

use byteorder::{ByteOrder, LittleEndian};

use flow_core::mem::PhysicalMemory;

pub fn find<T: PhysicalMemory>(mem: &mut T) -> Result<()> {
    let low16 = mem.read_physical_memory(0, 0x100000)?;

    // find x64 dtb in low stub < 1M
    find_x64_lowstub(&low16)?;

    // find x64 dtb in stub < 16M

    // find x86-pae dtb in stub < 16M

    // find x86 dtb in stub < 16M

    Ok(())
}

pub fn find_x64_lowstub(mem: &Vec<u8>) -> Result<()> {
    let chunk = mem
        .chunks_exact(0x1000)
        .skip(1)
        .filter(|c| (0xffffffffffff00ff & LittleEndian::read_u64(&c)) == 0x00000001000600E9) // start bytes
        .filter(|c| (0xfffff80000000003 & LittleEndian::read_u64(&c[0x70..])) == 0xfffff80000000000) // kernel entry
        .filter(|c| (0xffffff0000000fff & LittleEndian::read_u64(&c[0xA0..])) == 0) // pml4
        .nth(0)
        .ok_or(Error::new(ErrorKind::Other, "unable to find dtb in x64 lowstub"))?;

    // found!
    let va = LittleEndian::read_u64(&chunk[0x70..]);
    let dtb = LittleEndian::read_u64(&chunk[0xA0..]);
    println!("found x64 dtb in lowstub < 1M: va={:x} dtb={:x}", va, dtb);
    Ok(())
}

pub fn find_x64(mem: &Vec<u8>) -> Result<()> {
    Ok(())
}
