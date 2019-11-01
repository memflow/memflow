use log::{debug, trace};
use std::collections::HashMap;
use std::path::PathBuf;

use address::{Address, Length};
use mem::{VirtualRead};

use crate::kernel::StartBlock;

pub mod types;

// TODO: temporary
use std::ffi::CStr;
use std::os::raw::c_char;
// ...

// TODO: cache processes somewhat?
pub struct Windows {
    pub start_block: StartBlock,
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

    // TODO: store mem in win?
    // iterate over _EPROCESS structure
    pub fn process_iter<T: VirtualRead>(&mut self, mem: &mut T) {
        // TODO: --- test code ---
        let eproc = self.get_kernel_struct("_EPROCESS").unwrap();
        let offs_pid = eproc.get_field("UniqueProcessId").unwrap().offset;
        let offs_name = eproc.get_field("ImageFileName").unwrap().offset;
        let offs_links = eproc.get_field("ActiveProcessLinks").unwrap().offset;

        let offs_blink = self.get_kernel_struct("_LIST_ENTRY").unwrap().get_field("Blink").unwrap().offset;

        println!("offs_links: {:?}", offs_links);
        println!("offs_blink: {:?}", offs_blink);

        let mut eprocess_base = self.eprocess_base;
        loop {
            let pid = mem.virt_read_i32(self.start_block.arch,
                self.start_block.dtb,
                eprocess_base + Length::from(offs_pid)).unwrap();
            println!("pid of process: {}", pid);

            let namebuf = mem.virt_read_cstr(self.start_block.arch,
                self.start_block.dtb,
                eprocess_base + Length::from(offs_name), Length::from(16)).unwrap();

            //let rust_id = unsafe { CStr::from_ptr(namebuf.as_ptr()) };

            //let namecstr = CStr::from_bytes_with_nul(&namebuf).unwrap();
            println!("name of process: {:?}", namebuf);

            // read next entry
            eprocess_base = mem.virt_read_addr(
                self.start_block.arch,
                self.start_block.dtb,
                eprocess_base + Length::from(offs_links + offs_blink)).unwrap();
            if eprocess_base.is_null() {
                break;
            }

            eprocess_base -= Length::from(offs_links);
            if eprocess_base == self.eprocess_base {
                break;
            }

        }
        // TODO: -----------------
    }

    // iterate over kernel modules
    // TODO: ...
    pub fn module_iter() {
    }
}

#[derive(Clone)]
pub struct ProcessIterator {
    // ProcessIterator
    // store info about current process, eprocess for example
}

// TODO: should we borrow pe header here?
// TODO2: pdb should only be resolved for ms processes, in particular ntoskrnl!
// move pdb to Windows {} -> kernel_pdb or something
// also backjwards ref WinProcess in Windows for ntoskrnl
//impl WinProcess {
    /*
    pub fn from(base: Address, pe: &PE) -> Self {
        Self {
            base: base,
            size: Length::from(pe.size),
            pdb: cache::fetch_pdb(pe).ok(),
        }
    }
    */
//}
/*
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
*/