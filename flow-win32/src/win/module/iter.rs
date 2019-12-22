use crate::error::Result;

use flow_core::address::{Address, Length};

use flow_core::arch::{Architecture, ArchitectureTrait, InstructionSet};
use flow_core::mem::*;

use std::cell::RefCell;
use std::rc::Rc;

use super::Module;

use crate::win::process::ProcessModuleTrait;
use crate::win::unicode_string::VirtualReadUnicodeString;

pub struct ModuleIterator<
    T: ProcessModuleTrait + ArchitectureTrait + VirtualReadHelperFuncs + VirtualReadUnicodeString,
> {
    process: Rc<RefCell<T>>,
    process_arch: Architecture,
    first_peb_entry: Address,
    peb_entry: Address,
}

impl<T> ModuleIterator<T>
where
    T: ProcessModuleTrait + ArchitectureTrait + VirtualReadHelperFuncs + VirtualReadUnicodeString,
{
    pub fn new(process: Rc<RefCell<T>>) -> Result<Self> {
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

impl<T> Iterator for ModuleIterator<T>
where
    T: ProcessModuleTrait + ArchitectureTrait + VirtualReadHelperFuncs + VirtualReadUnicodeString,
{
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
