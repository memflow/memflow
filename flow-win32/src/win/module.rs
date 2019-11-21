use crate::error::{Error, Result};
use log::debug;

use crate::win::{Windows, types::PDB};
use crate::kernel::StartBlock;

use flow_core::address::{Address, Length};
use flow_core::mem::{VirtualRead};

use std::rc::Rc;
use std::cell::RefCell;

use widestring::U16CString;

use crate::win::process::Process;

pub struct ModuleIterator<'a, T: VirtualRead> {
    process: &'a mut Process<T>,
    first_module: Address,
    module_base: Address,
}

impl<'a, T: VirtualRead> ModuleIterator<'a, T> {
    pub fn new(process: &'a mut Process<T>) -> Result<Self> {
        let first_module = process.get_first_module()?;
        Ok(Self{
            process: process,
            first_module: first_module,
            module_base: first_module,
        })
    }
}

impl<'a, T: VirtualRead> Iterator for ModuleIterator<'a, T> {
    type Item = Module<T>;

    fn next(&mut self) -> Option<Module<T>> {
        // is module_base null?
        if self.module_base.is_null() {
            return None;
        }

        // copy memory for the lifetime of this function
        let dtb = self.process.get_dtb().unwrap(); // TODO: to option

        let memcp = self.process.mem.clone();
        let memory = &mut memcp.borrow_mut();

        let _list_entry_blink = self.process.get_offset("_LIST_ENTRY", "Blink").unwrap(); // TODO: err -> option

        // read next module entry (list_entry is first element in module)
        let mut next = memory.virt_read_addr(
            self.process.start_block.arch,
            dtb,
            self.module_base + _list_entry_blink).unwrap(); // TODO: convert to Option
    
        // if next process is 'system' again just null it
        if next == self.first_module {
            next = Address::null();
        }

        // return the previous process and set 'next' for next iter
        let cur = self.module_base;
        self.module_base = next;

        Some(Module::new(self.process, dtb, cur))
    }
}

// TODO: reference Process
pub struct Module<T: VirtualRead> {
    pub mem: Rc<RefCell<T>>,
    pub start_block: StartBlock,
    pub kernel_pdb: Option<PDB>, // TODO: refcell + shared access?
    pub dtb: Address,
    pub module_base: Address,
}

impl<T: VirtualRead> Module<T> {
    pub fn new(process: &Process<T>, dtb: Address, module_base: Address) -> Self {
        Self{
            mem: process.mem.clone(),
            start_block: process.start_block,
            kernel_pdb: process.kernel_pdb.clone(),
            dtb: dtb,
            module_base: module_base,
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

        let memory = &mut self.mem.borrow_mut();

        // TODO: x86 / wow64 version
        // if x64 && !wow64
        // TODO: access process
        let length = memory.virt_read_u16(
            self.start_block.arch,
            self.dtb,
            self.module_base + offs + Length::from(0))?;
        if length == 0 {
            return Err(Error::new("unable to read unicode string length"));
        }
        let buffer = memory.virt_read_addr(
            self.start_block.arch,
            self.dtb,
            self.module_base + offs + Length::from(8))?;
        if buffer.is_null() {
            return Err(Error::new("unable to read unicode string length"));
        }
        // else ...

        // buffer len > 4kb? ... abort
        if length % 2 != 0 {
            return Err(Error::new("unicode string length is not a multiple of two"));
        }

        // read buffer
        let mut content = memory.virt_read(
            self.start_block.arch,
            self.dtb,
            buffer,
            Length::from(length + 2))?;
        content[length as usize] = 0;
        content[length as usize + 1] = 0;

        // TODO: check length % 2 == 0

        let _content: Vec<u16> = unsafe { std::mem::transmute::<Vec<u8>, Vec<u16>>(content.into()) };
        Ok(U16CString::from_vec_with_nul(_content)?.to_string_lossy())
    }
}
