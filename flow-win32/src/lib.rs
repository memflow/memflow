use log::{info, trace, debug};

// TODO: custom errors
use std::io::Result;

use mem::{PhysicalRead, VirtualRead};

// TODO: move this in a seperate crate as a elf/pe/macho helper for pa/va
pub mod pe;

pub mod dtb;
pub mod ntos;

pub fn init<T: PhysicalRead + VirtualRead>(mem: &mut T) -> Result<()> {
    // TODO: add options to supply valid dtb

    // find dirtable base
    let dtb = dtb::find(mem)?;
    info!("dtb::find: arch={:?} va={:x} dtb={:x}", dtb.arch, dtb.va, dtb.dtb);

/*
    machine.cpu = Some(CPU{
        byte_order: ByteOrder::LittleEndian,
        arch: dtb.arch,
    })
*/

    // TODO: add option to supply a va hint
    // find ntoskrnl.exe base
    let ntos = ntos::find(mem, dtb)?;

    pe::test_read_pe(mem, dtb, ntos)?;

    // TODO: copy architecture and 

    Ok(())
}
