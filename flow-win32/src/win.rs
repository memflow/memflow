use std::path::PathBuf;

use address::{Address, Length};
use goblin::pe::PE;

use crate::dtb::DTB;
use crate::cache;

// TODO: cache processes somewhat?
pub struct Windows {
    pub dtb: DTB,
    pub kernel_base: Address,
    pub eproc_base: Address,
}

#[derive(Clone)]
pub struct WinProcess {
    pub base: Address,
    pub size: Length,
    pub pdb: Option<PathBuf>,
}

// TODO: should we borrow pe header here?
// TODO2: pdb should only be resolved for ms processes, in particular ntoskrnl!
// move pdb to Windows {} -> kernel_pdb or something
// also backjwards ref WinProcess in Windows for ntoskrnl
impl WinProcess {
    pub fn from(base: Address, pe: &PE) -> Self {
        Self {
            base: base,
            size: Length::from(pe.size),
            pdb: cache::fetch_pdb(pe).ok(),
        }
    }
}

// TODO: move to base
pub trait Process {
    fn pid() -> u64;
    fn name() -> String;
}

impl Process for WinProcess {
    fn pid() -> u64 {
        return 0;
    }

    fn name() -> String {
        return String::from("unknown");
    }
}

// TODO: move to base
pub trait ProcessList<T: Process> {
    fn process_list(&mut self) -> Vec<T>;
}

impl ProcessList<WinProcess> for Windows {
    fn process_list(&mut self) -> Vec<WinProcess> {
        let pl = Vec::new();
        pl
    }
}
