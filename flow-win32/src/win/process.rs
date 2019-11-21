use crate::error::Result;
use log::{info, debug};

use crate::win::{Windows, types::PDB};
use crate::kernel::StartBlock;

use flow_core::address::{Address, Length};
use flow_core::mem::{VirtualRead};

use std::rc::Rc;
use std::cell::RefCell;

use crate::win::module::ModuleIterator;

pub struct ProcessIterator<'a, T: VirtualRead> {
    win: &'a mut Windows<T>,
    first_eprocess: Address,
    eprocess: Address,
}

impl<'a, T: VirtualRead> ProcessIterator<'a, T> {
    pub fn new(win: &'a mut Windows<T>) -> Self {
        let eprocess = win.eprocess_base;
        Self{
            win: win,
            first_eprocess: eprocess,
            eprocess: eprocess,
        }
    }
}

impl<'a, T: VirtualRead> Iterator for ProcessIterator<'a, T> {
    type Item = Process<T>;

    fn next(&mut self) -> Option<Process<T>> {
        // is eprocess null (first iter, read err, sysproc)?
        if self.eprocess.is_null() {
            return None;
        }

        // copy memory for the lifetime of this function
        let memcp = self.win.mem.clone();
        let memory = &mut memcp.borrow_mut();

        // resolve offsets
        let _eprocess = self.win.kernel_pdb.clone()?.get_struct("_EPROCESS")?;
        let _list_entry = self.win.kernel_pdb.clone()?.get_struct("_LIST_ENTRY")?;

        let _eprocess_links = _eprocess.get_field("ActiveProcessLinks")?.offset;
        let _list_entry_blink = _list_entry.get_field("Blink")?.offset;

        // read next eprocess entry
        let mut next = memory.virt_read_addr(
            self.win.start_block.arch,
            self.win.start_block.dtb,
            self.eprocess + _eprocess_links + _list_entry_blink).unwrap(); // TODO: convert to Option
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

        Some(Process::new(self.win, cur))
    }
}

pub struct Process<T: VirtualRead> {
    pub mem: Rc<RefCell<T>>,
    pub start_block: StartBlock,
    pub kernel_pdb: Option<PDB>, // TODO: refcell + shared access?
    pub eprocess: Address,
}

// TODO: read/ret "ProcessInfo"
impl<T: VirtualRead> Process<T> {
    pub fn new(win: &Windows<T>, eprocess: Address) -> Self {
        Self{
            mem: win.mem.clone(),
            start_block: win.start_block,
            kernel_pdb: win.kernel_pdb.clone(),
            eprocess: eprocess,
        }
    }

    // TODO: macro? pub?
    pub fn get_offset(&mut self, strct: &str, field: &str) -> Result<Length> {
        let mut _pdb = self.kernel_pdb.as_mut().ok_or_else(|| "kernel pdb not found")?;
        let _strct = _pdb.get_struct(strct).ok_or_else(|| format!("{} not found", strct))?;
        let _field = _strct.get_field(field).ok_or_else(|| format!("{} not found", field))?;
        debug!("offset {}::{}={:x}", strct, field, _field.offset);
        Ok(_field.offset)
    }

    pub fn get_pid(&mut self) -> Result<i32> {
        let offs = self.get_offset("_EPROCESS", "UniqueProcessId")?;
        let memory = &mut self.mem.borrow_mut();
        Ok(memory.virt_read_i32(
            self.start_block.arch,
            self.start_block.dtb,
            self.eprocess + offs)?)
    }

    pub fn get_name(&mut self) -> Result<String> {
        let offs = self.get_offset("_EPROCESS", "ImageFileName")?;
        let memory = &mut self.mem.borrow_mut();
        Ok(memory.virt_read_cstr(
            self.start_block.arch,
            self.start_block.dtb,
            self.eprocess + offs,
            Length::from(16))?)
    }

    // TODO: dtb trait
    pub fn get_dtb(&mut self) -> Result<Address> {
        // _KPROCESS is the first entry in _EPROCESS
        let offs = self.get_offset("_KPROCESS", "DirectoryTableBase")?;
        let memory = &mut self.mem.borrow_mut();
        Ok(memory.virt_read_addr(
            self.start_block.arch,
            self.start_block.dtb,
            self.eprocess + offs)?)
    }

    pub fn get_wow64(&mut self) -> Result<Address> {
        let offs = self.get_offset("_EPROCESS", "WoW64Process")?;
        let memory = &mut self.mem.borrow_mut();
        Ok(memory.virt_read_addr(
            self.start_block.arch,
            self.start_block.dtb,
            self.eprocess + offs)?)
    }

    pub fn is_wow64(&mut self) -> Result<bool> {
        Ok(!self.get_wow64()?.is_null())
    }

    pub fn get_first_module(&mut self) -> Result<Address> {
        let wow64 = self.get_wow64()?;
        info!("wow64={:x}", wow64);

        let peb = match wow64.is_null() {
            true => {
                // x64
                info!("reading peb for x64 process");
                let offs = self.get_offset("_EPROCESS", "Peb")?;
                let memory = &mut self.mem.borrow_mut();
                memory.virt_read_addr(
                    self.start_block.arch,
                    self.start_block.dtb,
                    self.eprocess + offs)?
            },
            false => {
                // wow64 (first entry in wow64 struct = peb)
                info!("reading peb for wow64 process");
                let memory = &mut self.mem.borrow_mut();
                memory.virt_read_addr(
                    self.start_block.arch,
                    self.start_block.dtb,
                    wow64)?
            },
        };
        info!("peb={:x}", peb);

        // TODO: process.virt_read_addr based on wow64 or not
        // TODO: forward declare virtual read in process?

        let dtb = self.get_dtb()?;

        // TODO: move this print into get_offset or get_field
        let ldr_offs = self.get_offset("_PEB", "Ldr")?;
        let ldr_list_offs = self.get_offset("_PEB_LDR_DATA", "InLoadOrderModuleList")?;

        // read PPEB_LDR_DATA Ldr
        // addr_t peb_ldr = this->read_ptr(peb + this->mo_ldr);
        let peb_ldr = {
            let memory = &mut self.mem.borrow_mut();
            memory.virt_read_addr(
                self.start_block.arch,
                dtb,
                peb + ldr_offs)?
        };
        info!("peb_ldr={:x}", peb_ldr);

        // loop LIST_ENTRY InLoadOrderModuleList
        // addr_t first_module = this->read_ptr(peb_ldr + this->mo_ldr_list);
        let first_module = {
            let memory = &mut self.mem.borrow_mut();
            memory.virt_read_addr(
                self.start_block.arch,
                dtb,
                peb_ldr + ldr_list_offs)?
        };
        info!("first_module={:x}", first_module);
        Ok(first_module)
    }

    pub fn module_iter(&mut self) -> Result<ModuleIterator<'_, T>> {
        ModuleIterator::new(self)
    }
}
