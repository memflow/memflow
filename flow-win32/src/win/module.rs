use crate::error::{Error, Result};

use flow_core::*;

use flow_core::address::{Address, Length};

use flow_core::arch::{Architecture, InstructionSet};
use flow_core::mem::VirtualRead;

use std::cell::RefCell;
use std::rc::Rc;

use super::process::Process;
use super::unicode_string::VirtualReadUnicodeString;

pub struct ModuleIterator<T: VirtualRead> {
    process: Rc<RefCell<Process<T>>>,
    process_arch: Architecture,
    first_peb_entry: Address,
    peb_entry: Address,
}

impl<T: VirtualRead> ModuleIterator<T> {
    pub fn new(process: Rc<RefCell<Process<T>>>) -> Result<Self> {
        let first_peb_entry = process.borrow_mut().first_peb_entry()?;
        let arch = process.borrow_mut().arch()?;
        Ok(Self {
            process,
            process_arch: arch,
            first_peb_entry,
            peb_entry: first_peb_entry,
        })
    }
}

impl<T: VirtualRead> Iterator for ModuleIterator<T> {
    type Item = Module<T>;

    fn next(&mut self) -> Option<Module<T>> {
        // is peb_entry null?
        if self.peb_entry.is_null() {
            return None;
        }

        let process = &mut self.process.borrow_mut();

        //this is either 4 (x86) or 8 (x64)
        let _list_entry_blink = match self.process_arch.instruction_set {
            InstructionSet::X64 => Length::from(8),
            InstructionSet::X86 => Length::from(4),
            _ => return None,
        };

        // read next module entry (list_entry is first element in module)
        let mut next = process
            .virt_read_addr(self.peb_entry + _list_entry_blink)
            .unwrap(); // TODO: convert to Option

        // if next process is 'system' again just null it
        if next == self.first_peb_entry {
            next = Address::null();
        }

        // return the previous process and set 'next' for next iter
        let cur = self.peb_entry;
        self.peb_entry = next;

        Some(Module::new(self.process.clone(), cur))
    }
}

pub struct Module<T: VirtualRead> {
    pub process: Rc<RefCell<Process<T>>>,
    pub peb_entry: Address,
    pub module_base: Address,
    pub module_size: Length,
    pub module_name: String,
}

impl<T: VirtualRead> Clone for Module<T>
where
    Rc<RefCell<Process<T>>>: Clone,
    Address: Clone,
{
    fn clone(&self) -> Self {
        Self {
            process: self.process.clone(),
            peb_entry: self.peb_entry,
            module_base: self.module_base,
            module_size: self.module_size,
            module_name: self.module_name.clone(),
        }
    }
}

impl<T: VirtualRead> Module<T> {
    pub fn new(process: Rc<RefCell<Process<T>>>, peb_entry: Address) -> Self {
        Self {
            process,
            peb_entry,
            module_base: addr!(0),
            module_size: len!(0),
            module_name: String::from(""),
        }
    }

    // TODO: 0 should also result in an error
    pub fn base(&mut self) -> Result<Address> {
        if !self.module_base.is_null() {
            return Ok(self.module_base);
        }

        let process = &mut self.process.borrow_mut();
        let proc_arch = { process.arch()? };

        self.module_base = match proc_arch.instruction_set {
            InstructionSet::X64 => addr!(process.virt_read_u64(self.peb_entry + len!(0x30))?), // self.get_offset("_LDR_DATA_TABLE_ENTRY", "DllBase")?
            InstructionSet::X86 => addr!(process.virt_read_u32(self.peb_entry + len!(0x18))?),
            _ => return Err(Error::new("invalid process architecture")),
        };
        Ok(self.module_base)
    }

    // TODO: 0 should also result in an error
    pub fn size(&mut self) -> Result<Length> {
        if !self.module_size.is_zero() {
            return Ok(self.module_size);
        }

        let process = &mut self.process.borrow_mut();
        let proc_arch = { process.arch()? };

        self.module_size = match proc_arch.instruction_set {
            InstructionSet::X64 => len!(process.virt_read_u64(self.peb_entry + len!(0x40))?), // self.get_offset("_LDR_DATA_TABLE_ENTRY", "SizeOfImage")?
            InstructionSet::X86 => len!(process.virt_read_u32(self.peb_entry + len!(0x20))?),
            _ => return Err(Error::new("invalid process architecture")),
        };
        Ok(self.module_size)
    }

    pub fn name(&mut self) -> Result<String> {
        if self.module_name != "" {
            return Ok(self.module_name.clone());
        }

        let process = &mut self.process.borrow_mut();
        let cpu_arch = { process.win.borrow().start_block.arch };
        let proc_arch = { process.arch()? };
        let dtb = process.dtb()?;

        let offs = match proc_arch.instruction_set {
            InstructionSet::X64 => len!(0x58), // self.get_offset("_LDR_DATA_TABLE_ENTRY", "BaseDllName")?,
            InstructionSet::X86 => len!(0x2C),
            _ => return Err(Error::new("invalid process architecture")),
        };

        let win = process.win.borrow();
        let mem = &mut win.mem.borrow_mut();

        // x64 = x64 && !wow64
        // TODO: wrap virt_read_unicode_string in process::virt_read
        self.module_name =
            mem.virt_read_unicode_string(cpu_arch, proc_arch, dtb, self.peb_entry + offs)?;
        Ok(self.module_name.clone())
    }
}
