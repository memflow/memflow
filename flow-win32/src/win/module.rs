use crate::error::{Error, Result};
use log::debug;

use crate::kernel::StartBlock;
use super::{types::PDB, Windows};

use flow_core::address::{Address, Length};
use flow_core::arch::{self, Architecture, InstructionSet};
use flow_core::mem::VirtualRead;

use std::cell::RefCell;
use std::rc::Rc;

use super::process::Process;
use super::unicode_string::VirtualReadUnicodeString;

use super::process::virt_read::ProcessRead;

pub struct ModuleIterator<T: VirtualRead> {
    process: Rc<RefCell<Process<T>>>,
    process_arch: Architecture,
    first_module: Address,
    module_base: Address,
}

impl<T: VirtualRead> ModuleIterator<T> {
    pub fn new(process: Rc<RefCell<Process<T>>>) -> Result<Self> {
        let first_module = process.borrow_mut().get_first_module()?;
        let arch = process.borrow_mut().get_process_arch()?;
        Ok(Self {
            process: process,
            process_arch: arch,
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

        let process = &mut self.process.borrow_mut();

        //this is either 4 (x86) or 8 (x64)
        let _list_entry_blink = match self.process_arch.instruction_set {
            InstructionSet::X64 => Length::from(8),
            InstructionSet::X86 => Length::from(4),
            _ => return None,
        };

        // read next module entry (list_entry is first element in module)
        let mut next = process
            .virt_read_addr(self.module_base + _list_entry_blink)
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
    fn get_offset(&mut self, strct: &str, field: &str) -> Result<Length> {
        let process = &mut self.process.borrow_mut();
        process.get_offset(strct, field)
    }

    pub fn get_module_base(self) -> Address {
        self.module_base
    }

    pub fn get_name(&mut self) -> Result<String> {
        // TODO: this is architecture dependent
        //let base_offs = process.get_offset("_LDR_DATA_TABLE_ENTRY", "DllBase")?;
        //let size_offs = process.get_offset("_LDR_DATA_TABLE_ENTRY", "SizeOfImage")?;

        let process = &mut self.process.borrow_mut();
        let cpu_arch = { process.win.borrow().start_block.arch };
        let proc_arch = { process.get_process_arch()? };
        let dtb = process.get_dtb()?;

        let offs = match proc_arch.instruction_set {
            InstructionSet::X64 => Length::from(0x58), // self.get_offset("_LDR_DATA_TABLE_ENTRY", "BaseDllName")?,
            InstructionSet::X86 => Length::from(0x2C),
            _ => return Err(Error::new("invalid process architecture")),
        };

        let win = process.win.borrow();
        let mem = &mut win.mem.borrow_mut();

        // x64 = x64 && !wow64
        mem.virt_read_unicode_string(
            cpu_arch,
            proc_arch,
            dtb,
            self.module_base + offs)
    }
}
