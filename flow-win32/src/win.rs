use address::Address;
use goblin::pe::PE;

use crate::dtb::DTB;

// TODO: cache processes somewhat?
pub struct Windows {
    pub dtb: DTB,
    pub kernel_base: Address,
    pub eproc_base: Address,
}

#[derive(Clone, Copy)]
pub struct WinProcess {
    //pub pe: PE,
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
