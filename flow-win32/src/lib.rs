// TODO: custom errors
use std::io::{Error, ErrorKind, Result};

use flow_core::machine::Machine;

pub mod dtb;
pub mod ntos;

pub fn init(machine: &mut Machine) -> Result<()> {
    // TODO: add options to supply valid dtb

    // find dirtable base
    let dtb = dtb::find(machine)?;
    println!("dtb::find(): arch={:?} va={:x} dtb={:x}", dtb.arch, dtb.va, dtb.dtb);

    // TODO: add option to supply a va hint
    // find ntoskrnl.exe base
    let ntos = ntos::find(machine, dtb)?;

    Ok(())
}
