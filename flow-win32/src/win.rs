use log::{debug, trace};
use std::collections::HashMap;
use std::path::PathBuf;

use address::{Address, Length};

use crate::kernel::KernelStubInfo;

pub mod types;

// TODO: cache processes somewhat?
pub struct Windows {
    pub kernel_stub_info: KernelStubInfo,
    pub kernel_base: Address,
    pub eprocess_base: Address,

    pub kernel_pdb: Option<PathBuf>,
    pub kernel_structs: HashMap<String, types::Struct>,
}

impl Windows {
    pub fn get_kernel_struct<'a>(&'a mut self, name: &str) -> Option<types::Struct> {
        match self.kernel_structs.get(name) {
            Some(s) => return Some(s.clone()),
            None => trace!("struct {} not found in cache", name),
        }

        // TODO: ?
        match self.kernel_pdb {
            Some(_) => (),
            None => {
                debug!(
                    "unable to resolve kernel_struct {} since pdb was not found",
                    name
                );
                return None;
            }
        }

        let pdb = self.kernel_pdb.clone().unwrap();
        match types::Struct::from(pdb, name) {
            Ok(s) => {
                self.kernel_structs.insert(String::from(name), s.clone());
                return Some(s);
            }
            Err(e) => trace!("struct {} not found: {:?}", name, e),
        }
        None
    }
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
    /*
    pub fn from(base: Address, pe: &PE) -> Self {
        Self {
            base: base,
            size: Length::from(pe.size),
            pdb: cache::fetch_pdb(pe).ok(),
        }
    }
    */
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
