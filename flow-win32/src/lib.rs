use crate::error::Result;
use log::info;
use flow_core::mem::{PhysicalRead, VirtualRead};

pub mod cache;
pub mod error;
pub mod kernel;
pub mod pe;
pub mod win;

use win::{Windows, types::PDB};

use std::rc::Rc;
use std::cell::RefCell;

/*
Options:
- supply cr3 (dtb)
- supply kernel hint
- supply pdb
- supply kernel offsets for basic structs (dumped windbg maybe)
*/

type Memory<T> = Rc<RefCell<T>>;

// TODO: impl Windows {}
pub fn init<T: PhysicalRead + VirtualRead>(mem: Memory<T>) -> Result<Windows<T>> {
    // copy rc and borrow it temporarily
    let memcp = mem.clone();
    let memory: &mut T = &mut memcp.borrow_mut();

    // find dirtable base
    let start_block = kernel::lowstub::find(memory)?;
    info!(
        "arch={:?} va={:x} dtb={:x}",
        start_block.arch, start_block.va, start_block.dtb
    );

    // find ntoskrnl.exe base
    let kernel_base = kernel::ntos::find(memory, &start_block)?;
    info!("kernel_base={:x}", kernel_base);

    // system eprocess -> find
    let eprocess_base = kernel::sysproc::find(memory, &start_block, kernel_base)?;
    info!("eprocess_base={:x}", eprocess_base);

    // TODO: add a module like sysproc/ntoskrnl/etc which will fetch pdb with various fallbacks and return early here
    // TODO: create fallback thingie which implements hardcoded offsets
    // TODO: create fallback which parses C struct from conf file + manual pdb
    // TODO: add class wrapper to Windows struct

    // grab pdb
    let kernel_pdb = match cache::fetch_pdb_from_mem(memory, &start_block, kernel_base) {
        Ok(p) => {
            info!("valid kernel_pdb found: {:?}", p);
            Some(Rc::new(RefCell::new(PDB::new(p))))
        },
        Err(e) => {
            info!("unable to fetch pdb for ntoskrnl: {:?}", e);
            None
        }
    };

    Ok(Windows {
        mem: mem,
        start_block: start_block,
        kernel_base: kernel_base,
        eprocess_base: eprocess_base,
        kernel_pdb: kernel_pdb,
    })
}
