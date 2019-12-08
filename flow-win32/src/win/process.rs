pub mod virt_read;

use crate::error::{Error, Result};
use log::{debug, info};

use crate::kernel::StartBlock;
use super::{types::PDB, Windows};

use flow_core::address::{Address, Length};
use flow_core::mem::VirtualRead;
use flow_core::arch::{InstructionSet, Architecture};

use std::cell::RefCell;
use std::rc::Rc;

use super::module::ModuleIterator;

use virt_read::ProcessRead;

pub struct ProcessIterator<T: VirtualRead> {
    win: Rc<RefCell<Windows<T>>>,
    first_eprocess: Address,
    eprocess: Address,
}

impl<T: VirtualRead> ProcessIterator<T> {
    pub fn new(win: Rc<RefCell<Windows<T>>>) -> Self {
        let eprocess = win.borrow().eprocess_base;
        Self {
            win: win,
            first_eprocess: eprocess,
            eprocess: eprocess,
        }
    }
}

impl<T: VirtualRead> Iterator for ProcessIterator<T> {
    type Item = Process<T>;

    fn next(&mut self) -> Option<Process<T>> {
        // is eprocess null (first iter, read err, sysproc)?
        if self.eprocess.is_null() {
            return None;
        }

        // copy memory for the lifetime of this function
        let win = self.win.borrow();
        let start_block = win.start_block;
        let kernel_pdb = &mut win.kernel_pdb.as_ref()?.borrow_mut();

        let memory = &mut win.mem.borrow_mut();

        // resolve offsets
        let _eprocess = kernel_pdb.get_struct("_EPROCESS")?;
        let _eprocess_links = _eprocess.get_field("ActiveProcessLinks")?.offset;

        let _list_entry = kernel_pdb.get_struct("_LIST_ENTRY")?;
        let _list_entry_blink = _list_entry.get_field("Blink")?.offset;

        // read next eprocess entry
        let mut next = memory
            .virt_read_addr(
                start_block.arch,
                start_block.dtb,
                self.eprocess + _eprocess_links + _list_entry_blink,
            )
            .unwrap(); // TODO: convert to Option
        if !next.is_null() {
            next -= _eprocess_links;
        }

        // if next process is 'system' again just null it
        if next == self.first_eprocess {
            next = Address::null();
        }

        // return the previous process and set 'next' for next iter
        let cur = self.eprocess;
        self.eprocess = next;

        Some(Process::new(self.win.clone(), cur))
    }
}

pub struct Process<T: VirtualRead> {
    pub win: Rc<RefCell<Windows<T>>>,
    pub eprocess: Address,
}

impl<T: VirtualRead> Clone for Process<T>
where
    Rc<RefCell<T>>: Clone,
    Address: Clone,
{
    fn clone(&self) -> Self {
        Self {
            win: self.win.clone(),
            eprocess: self.eprocess.clone(),
        }
    }
}

// TODO: read/ret "ProcessInfo"
impl<T: VirtualRead> Process<T> {
    pub fn new(win: Rc<RefCell<Windows<T>>>, eprocess: Address) -> Self {
        Self {
            win: win,
            eprocess: eprocess,
        }
    }

    // TODO: macro? pub?
    pub fn get_offset(&mut self, strct: &str, field: &str) -> Result<Length> {
        let win = &mut self.win.borrow_mut();
        let mut _pdb = win
            .kernel_pdb
            .as_ref()
            .ok_or_else(|| "kernel pdb not found")?
            .borrow_mut();
        let _strct = _pdb
            .get_struct(strct)
            .ok_or_else(|| format!("{} not found", strct))?;
        let _field = _strct
            .get_field(field)
            .ok_or_else(|| format!("{} not found", field))?;
        debug!("offset {}::{}={:x}", strct, field, _field.offset);
        Ok(_field.offset)
    }

    pub fn get_pid(&mut self) -> Result<i32> {
        let offs = self.get_offset("_EPROCESS", "UniqueProcessId")?;
        let win = self.win.borrow();
        let start_block = win.start_block;
        let mem = &mut win.mem.borrow_mut();
        Ok(mem.virt_read_i32(start_block.arch, start_block.dtb, self.eprocess + offs)?)
    }

    pub fn get_name(&mut self) -> Result<String> {
        let offs = self.get_offset("_EPROCESS", "ImageFileName")?;
        let win = self.win.borrow();
        let start_block = win.start_block;
        let mem = &mut win.mem.borrow_mut();
        Ok(mem.virt_read_cstr(
            start_block.arch,
            start_block.dtb,
            self.eprocess + offs,
            Length::from(16),
        )?)
    }

    // TODO: dtb trait
    pub fn get_dtb(&mut self) -> Result<Address> {
        // _KPROCESS is the first entry in _EPROCESS
        let offs = self.get_offset("_KPROCESS", "DirectoryTableBase")?;
        let win = self.win.borrow();
        let start_block = win.start_block;
        let mem = &mut win.mem.borrow_mut();
        Ok(mem.virt_read_addr(start_block.arch, start_block.dtb, self.eprocess + offs)?)
    }

    pub fn get_wow64(&mut self) -> Result<Address> {
        let offs = self.get_offset("_EPROCESS", "WoW64Process")?;
        let win = self.win.borrow();
        let start_block = win.start_block;
        let mem = &mut win.mem.borrow_mut();
        Ok(mem.virt_read_addr(start_block.arch, start_block.dtb, self.eprocess + offs)?)
    }

    pub fn is_wow64(&mut self) -> Result<bool> {
        Ok(!self.get_wow64()?.is_null())
    }

    pub fn get_process_arch(&mut self) -> Result<Architecture> {
        // TODO: if x64 && !wow64
        if !self.is_wow64()? {
            Ok(Architecture::from(InstructionSet::X64))
        } else {
            Ok(Architecture::from(InstructionSet::X86))
        }
    }

    pub fn get_first_peb_entry(&mut self) -> Result<Address> {
        let wow64 = self.get_wow64()?;
        info!("wow64={:x}", wow64);

        let start_block = {
            let win = self.win.borrow();
            win.start_block
        };

        let peb = match wow64.is_null() {
            true => {
                // x64
                info!("reading peb for x64 process");
                let offs = self.get_offset("_EPROCESS", "Peb")?;
                let win = self.win.borrow();
                let mem = &mut win.mem.borrow_mut();
                mem.virt_read_addr(start_block.arch, start_block.dtb, self.eprocess + offs)?
            }
            false => {
                // wow64 (first entry in wow64 struct = peb)
                info!("reading peb for wow64 process");
                let win = self.win.borrow();
                let mem = &mut win.mem.borrow_mut();
                mem.virt_read_addr(start_block.arch, start_block.dtb, wow64)?
            }
        };
        info!("peb={:x}", peb);

        // TODO: process.virt_read_addr based on wow64 or not
        // TODO: forward declare virtual read in process?
        // TODO: use process architecture agnostic wrapper from here!

        // process architecture agnostic offsets
        let proc_arch = self.get_process_arch()?;

        let ldr_offs = match proc_arch.instruction_set {
            InstructionSet::X64 => Length::from(0x18), // self.get_offset("_PEB", "Ldr")?,
            InstructionSet::X86 => Length::from(0xC),
            _ => return Err(Error::new("invalid process architecture")),
        };

        let ldr_list_offs = match proc_arch.instruction_set {
            InstructionSet::X64 => Length::from(0x10), // self.get_offset("_PEB_LDR_DATA", "InLoadOrderModuleList")?,
            InstructionSet::X86 => Length::from(0xC),
            _ => return Err(Error::new("invalid process architecture")),
        };

        // read PPEB_LDR_DATA Ldr
        // addr_t peb_ldr = this->read_ptr(peb + this->mo_ldr);
        let peb_ldr = self.virt_read_addr(peb + ldr_offs)?;
        info!("peb_ldr={:x}", peb_ldr);

        // loop LIST_ENTRY InLoadOrderModuleList
        // addr_t first_module = this->read_ptr(peb_ldr + this->mo_ldr_list);
        let first_module = self.virt_read_addr(peb_ldr + ldr_list_offs)?;
        info!("first_module={:x}", first_module);
        Ok(first_module)
    }

    // module_iter will explicitly clone self and feed it into an iterator
    pub fn module_iter(&self) -> Result<ModuleIterator<T>> {
        let rc = Rc::new(RefCell::new(self.clone()));
        ModuleIterator::new(rc)
    }
}
