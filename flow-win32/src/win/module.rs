use crate::error::{Error, Result};
use log::debug;

use crate::kernel::StartBlock;
use crate::win::{types::PDB, Windows};

use flow_core::address::{Address, Length};
use flow_core::mem::VirtualRead;

use std::cell::RefCell;
use std::rc::Rc;

use widestring::U16CString;

use crate::win::process::Process;

pub struct ModuleIterator<T: VirtualRead> {
    process: Rc<RefCell<Process<T>>>,
    first_module: Address,
    module_base: Address,
}

impl<T: VirtualRead> ModuleIterator<T> {
    pub fn new(process: Rc<RefCell<Process<T>>>) -> Result<Self> {
        let first_module = process.borrow_mut().get_first_module()?;
        //let first_module = process.get_first_module()?;
        Ok(Self {
            process: process,
            first_module: first_module,
            module_base: first_module,
        })
    }
}

impl<T: VirtualRead> Iterator for ModuleIterator<T> {
    type Item = Module<T>;

    fn next(&mut self) -> Option<Module<T>> {
        // is module_base null?
        if self.module_base.is_null() {
            return None;
        }

        // borrow process
        let process = &mut self.process.borrow_mut();

        // copy memory for the lifetime of this function
        let start_block = { process.win.borrow().start_block };
        let dtb = process.get_dtb().unwrap(); // TODO: to option

        let _list_entry_blink = process.get_offset("_LIST_ENTRY", "Blink").unwrap(); // TODO: err -> option

        //let memcp = process.mem.clone();
        let win = process.win.borrow();
        let mem = &mut win.mem.borrow_mut();

        // read next module entry (list_entry is first element in module)
        let mut next = mem
            .virt_read_addr(start_block.arch, dtb, self.module_base + _list_entry_blink)
            .unwrap(); // TODO: convert to Option

        // if next process is 'system' again just null it
        if next == self.first_module {
            next = Address::null();
        }

        // return the previous process and set 'next' for next iter
        let cur = self.module_base;
        self.module_base = next;

        Some(Module::new(self.process.clone(), cur))
    }
}

pub struct Module<T: VirtualRead> {
    pub process: Rc<RefCell<Process<T>>>,
    pub module_base: Address,
}

impl<T: VirtualRead> Clone for Module<T>
where
    Rc<RefCell<Process<T>>>: Clone,
    Address: Clone,
{
    fn clone(&self) -> Self {
        Self {
            process: self.process.clone(),
            module_base: self.module_base.clone(),
        }
    }
}

impl<T: VirtualRead> Module<T> {
    pub fn new(process: Rc<RefCell<Process<T>>>, module_base: Address) -> Self {
        Self {
            process: process,
            module_base: module_base,
        }
    }

    // TODO: macro? pub?
    pub fn get_offset(&mut self, strct: &str, field: &str) -> Result<Length> {
        let process = &mut self.process.borrow_mut();
        process.get_offset(strct, field)
    }

    pub fn get_name(&mut self) -> Result<String> {
        /*
        typedef struct _windows_unicode_string32 {
            uint16_t length;
            uint16_t maximum_length;
            uint32_t pBuffer; // pointer to string contents
        } __attribute__((packed)) win32_unicode_string_t;

        typedef struct _windows_unicode_string64 {
            uint16_t length;
            uint16_t maximum_length;
            uint32_t padding; // align pBuffer
            uint64_t pBuffer; // pointer to string contents
        } __attribute__((packed)) win64_unicode_string_t;
        */

        // TODO: this is architecture dependent
        //let base_offs = process.get_offset("_LDR_DATA_TABLE_ENTRY", "DllBase")?;
        //let size_offs = process.get_offset("_LDR_DATA_TABLE_ENTRY", "SizeOfImage")?;

        let offs = self.get_offset("_LDR_DATA_TABLE_ENTRY", "BaseDllName")?;

        let process = &mut self.process.borrow_mut();
        let start_block = { process.win.borrow().start_block };
        let dtb = process.get_dtb()?;

        let win = process.win.borrow();
        let mem = &mut win.mem.borrow_mut();

        // TODO: x86 / wow64 version
        // if x64 && !wow64
        // TODO: access process
        let length = mem.virt_read_u16(
            start_block.arch,
            dtb,
            self.module_base + offs + Length::from(0),
        )?;
        if length == 0 {
            return Err(Error::new("unable to read unicode string length"));
        }
        let buffer = mem.virt_read_addr(
            start_block.arch,
            dtb,
            self.module_base + offs + Length::from(8),
        )?;
        if buffer.is_null() {
            return Err(Error::new("unable to read unicode string length"));
        }
        // else ...

        // buffer len > 4kb? ... abort
        if length % 2 != 0 {
            return Err(Error::new("unicode string length is not a multiple of two"));
        }

        // read buffer
        let mut content = mem.virt_read(start_block.arch, dtb, buffer, Length::from(length + 2))?;
        content[length as usize] = 0;
        content[length as usize + 1] = 0;

        // TODO: check length % 2 == 0

        let _content: Vec<u16> =
            unsafe { std::mem::transmute::<Vec<u8>, Vec<u16>>(content.into()) };
        Ok(U16CString::from_vec_with_nul(_content)?.to_string_lossy())
    }
}
