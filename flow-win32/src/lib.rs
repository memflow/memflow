use log::{debug, info, trace};

// TODO: custom errors
use std::io::Result;

use mem::{PhysicalRead, VirtualRead};

// TODO: move this in a seperate crate as a elf/pe/macho helper for pa/va
pub mod pe;

pub mod dtb;
pub mod ntos;
pub mod sysproc;
pub mod cache;
pub mod win;

use win::{Windows, ProcessList};

pub fn init<T: PhysicalRead + VirtualRead>(mem: &mut T) -> Result<Windows> {
    // TODO: add options to supply valid dtb

    // find dirtable base
    let dtb = dtb::find(mem)?;
    info!(
        "dtb::find: arch={:?} va={:x} dtb={:x}",
        dtb.arch, dtb.va, dtb.dtb
    );

    /*
        machine.cpu = Some(CPU{
            byte_order: ByteOrder::LittleEndian,
            arch: dtb.arch,
        })
    */

    // TODO: add option to supply a va hint
    // find ntoskrnl.exe base
    let ntos = ntos::find(mem, dtb)?;

    // system eprocess -> find
    let sysproc = sysproc::find(mem, dtb, ntos)?;

    // PsLoadedModuleList / KDBG -> find

    // pdb, winreg?

    //pe::test_read_pe(mem, dtb, ntos)?;

    // TODO: copy architecture and

    let mut win = Windows {
        dtb: dtb,
        kernel_base: ntos,
        eproc_base: sysproc,
    };
    let list = win.process_list();
    Ok(win)
}
