use log::{debug, info, trace};

// TODO: custom errors
use std::io::Result;

use mem::{PhysicalRead, VirtualRead};

// TODO: move this in a seperate crate as a elf/pe/macho helper for pa/va
pub mod pe;

pub mod cache;
pub mod dtb;
pub mod ntos;
pub mod sysproc;
pub mod win;

use win::{ProcessList, Windows};

pub fn init<T: PhysicalRead + VirtualRead>(mem: &mut T) -> Result<Windows> {
    // TODO: add options to supply valid dtb

    // find dirtable base
    let dtb = dtb::find(mem)?;
    info!("arch={:?} va={:x} dtb={:x}", dtb.arch, dtb.va, dtb.dtb);

    /*
        machine.cpu = Some(CPU{
            byte_order: ByteOrder::LittleEndian,
            arch: dtb.arch,
        })
    */

    // TODO: add option to supply a va hint
    // find ntoskrnl.exe base
    let ntos = ntos::find(mem, dtb)?;
    info!("ntos={:x}", ntos);

    // try to fetch pdb
    //let pdb = cache::fetch_pdb(pe)?;

    // system eprocess -> find
    let sysproc = sysproc::find(mem, dtb, ntos)?;
    info!("sysproc={:x}", sysproc);

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
