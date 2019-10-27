use crate::error::{Error, Result};

use log::{debug, info, trace};

// TODO: custom errors
use std::collections::HashMap;

use mem::{PhysicalRead, VirtualRead};

pub mod error;

// TODO: move this in a seperate crate as a elf/pe/macho helper for pa/va
pub mod pe;

pub mod cache;
pub mod dtb;
pub mod ntos;
pub mod sysproc;
pub mod win;

use win::{ProcessList, Windows};

/*
Options:
- supply cr3
- supply kernel hint
- supply pdb
- supply kernel offsets for basic structs (dumped windbg maybe)
*/

// TODO: impl Windows {}
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
    let kernel_base = ntos::find(mem, dtb)?;
    info!("ntos={:x}", kernel_base);

    // try to fetch pdb
    //let pdb = cache::fetch_pdb(pe)?;

    // system eprocess -> find
    let sysproc = sysproc::find(mem, dtb, kernel_base)?;
    info!("sysproc={:x}", sysproc);

    // grab pdb
    // TODO: new func or something in Windows impl
    let kernel_pdb = match cache::fetch_pdb_from_mem(mem, &dtb, kernel_base) {
        Ok(p) => Some(p),
        Err(e) => {
            info!("unable to fetch pdb from memory: {:?}", e);
            None
        }
    };

    println!("kernel_pdb: {:?}", kernel_pdb.clone().unwrap());

    let mut win = Windows {
        dtb: dtb,
        kernel_base: kernel_base,
        eproc_base: sysproc,
        kernel_pdb: kernel_pdb,
        kernel_structs: HashMap::new(),
    };

    // TODO: create fallback thingie which implements hardcoded offsets
    // TODO: create fallback which parses C struct from conf file + manual pdb
    // TODO: add class wrapper to Windows struct
    //let pdb = ; // TODO: add manual pdb option
    //let class = types::Struct::from(pdb, "_EPROCESS").unwrap();
    println!(
        "_EPROCESS::UniqueProcessId: {:?}",
        win.get_kernel_struct("_EPROCESS")
            .unwrap()
            .get_field("UniqueProcessId")
    );

    // PsLoadedModuleList / KDBG -> find

    // pdb, winreg?

    //pe::test_read_pe(mem, dtb, ntos)?;

    // TODO: copy architecture and

    let list = win.process_list();
    Ok(win)
}
