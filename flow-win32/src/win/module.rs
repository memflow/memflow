pub mod iter;
pub use iter::ModuleIterator;

use crate::error::{Error, Result};

use flow_core::*;

use flow_core::address::{Address, Length};

use flow_core::arch::{InstructionSet, SystemArchitecture};
use flow_core::mem::*;

use std::cell::RefCell;
use std::rc::Rc;

use crate::win::process::ProcessTrait;
use crate::win::unicode_string::VirtualReadUnicodeString;

pub struct Module<
    T: ProcessTrait + SystemArchitecture + VirtualReadHelperFuncs + VirtualReadUnicodeString,
> {
    pub process: Rc<RefCell<T>>,
    pub peb_entry: Address,
    pub module_base: Address,
    pub module_size: Length,
    pub module_name: String,
}

impl<T> Clone for Module<T>
where
    T: ProcessTrait + SystemArchitecture + VirtualReadHelperFuncs + VirtualReadUnicodeString,
    Rc<RefCell<T>>: Clone,
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

impl<T> Module<T>
where
    T: ProcessTrait + SystemArchitecture + VirtualReadHelperFuncs + VirtualReadUnicodeString,
{
    pub fn new(process: Rc<RefCell<T>>, peb_entry: Address) -> Self {
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
        let proc_arch = { process.arch()? };

        let offs = match proc_arch.instruction_set {
            InstructionSet::X64 => len!(0x58), // self.get_offset("_LDR_DATA_TABLE_ENTRY", "BaseDllName")?,
            InstructionSet::X86 => len!(0x2C),
            _ => return Err(Error::new("invalid process architecture")),
        };

        // x64 = x64 && !wow64
        self.module_name = process.virt_read_unicode_string(self.peb_entry + offs)?;
        Ok(self.module_name.clone())
    }
}

/*
impl Module<T>
where T: ProcessTrait + SystemArchitecture + VirtualReadHelperFuncs + VirtualReadUnicodeString
{
    // export_iter()
    // export(...)
    // section_iter()
    // section(...)
    // signature(...)
    // ...?

    pub fn export(&self, name: &str) -> Result<()> {
        // TODO: cache pe header
    }

    // TODO: implement caching
    fn try_read_pe_header() -> Result<> {
    }
}
*/
