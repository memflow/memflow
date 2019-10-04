// TODO: custom errors
use std::io::Result;

use flow_core::mem::{PhysicalRead, VirtualRead};

pub mod dtb;
pub mod ntos;

pub fn init<T: PhysicalRead + VirtualRead>(mem: &mut T) -> Result<()> {
    // TODO: add options to supply valid dtb

    // find dirtable base
    let dtb = dtb::find(mem)?;
    println!("dtb::find(): arch={:?} va={:x} dtb={:x}", dtb.arch, dtb.va, dtb.dtb);

/*
    machine.cpu = Some(CPU{
        byte_order: ByteOrder::LittleEndian,
        arch: dtb.arch,
    })
*/

    // TODO: add option to supply a va hint
    // find ntoskrnl.exe base
    let ntos = ntos::find(mem, dtb)?;

    // TODO: copy architecture and 

    Ok(())
}
