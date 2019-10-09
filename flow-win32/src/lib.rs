use log::{info, trace, debug};

// TODO: custom errors
use std::io::Result;

use mem::{PhysicalRead, VirtualRead};

// TODO: move this in a seperate crate as a elf/pe/macho helper for pa/va
pub mod pe;

pub mod dtb;
pub mod ntos;

// TODO: refactor/move - this is just temporary
use address::Address;
pub struct Windows {
    pub dtb: dtb::DTB,
    pub kernel_base: Address,
}

pub fn init<T: PhysicalRead + VirtualRead>(mem: &mut T) -> Result<Windows> {
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

    Ok(Windows {
        dtb: dtb,
        kernel_base: ntos,
    })
}
