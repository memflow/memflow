use crate::error::Result;
use log::info;
use mem::{PhysicalRead, VirtualRead};
use std::collections::HashMap;

pub mod cache;
pub mod error;
pub mod kernel;
pub mod pe;
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
    let start_block = kernel::lowstub::find(mem)?;
    info!(
        "arch={:?} va={:x} dtb={:x}",
        start_block.arch, start_block.va, start_block.dtb
    );

    /*
        machine.cpu = Some(CPU{
            byte_order: ByteOrder::LittleEndian,
            arch: dtb.arch,
        })
    */

    // TODO: add option to supply a va hint
    // find ntoskrnl.exe base
    let kernel_base = kernel::ntos::find(mem, &start_block)?;
    info!("kernel_base={:x}", kernel_base);

    // try to fetch pdb
    //let pdb = cache::fetch_pdb(pe)?;

    // system eprocess -> find
    let eprocess_base = kernel::sysproc::find(mem, &start_block, kernel_base)?;
    info!("eprocess_base={:x}", eprocess_base);

    // TODO: add a module like sysproc/ntoskrnl/etc which will fetch pdb with various fallbacks and return early here
    // grab pdb
    // TODO: new func or something in Windows impl
    let kernel_pdb = match cache::fetch_pdb_from_mem(mem, &start_block, kernel_base) {
        Ok(p) => {
            info!("valid kernel_pdb found: {:?}", p);
            Some(p)
        },
        Err(e) => {
            info!("unable to fetch pdb for ntoskrnl: {:?}", e);
            None
        }
    };

    let mut win = Windows {
        start_block: start_block,
        kernel_base: kernel_base,
        eprocess_base: eprocess_base,
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
