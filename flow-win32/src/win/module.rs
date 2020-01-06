pub mod iter;
pub use iter::*;

pub mod export;
pub use export::*;

pub mod section;
pub use section::*;

use crate::error::{Error, Result};

use std::cell::RefCell;
use std::rc::Rc;

use flow_core::address::{Address, Length};
use flow_core::arch::{ArchitectureTrait, InstructionSet};
use flow_core::mem::*;
use flow_core::process::ModuleTrait;
use flow_core::*;

use crate::win::process::ProcessModuleTrait;
use crate::win::unicode_string::VirtualReadUnicodeString;

use pelite::{self, pe64::exports, PeView};

pub struct Module<
    T: ProcessModuleTrait + ArchitectureTrait + VirtualReadHelperFuncs + VirtualReadUnicodeString,
> {
    pub process: Rc<RefCell<T>>,
    pub peb_entry: Address,
    pub module_base: Address,
    pub module_size: Length,
    pub module_name: String,
}

impl<T> Clone for Module<T>
where
    T: ProcessModuleTrait + ArchitectureTrait + VirtualReadHelperFuncs + VirtualReadUnicodeString,
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
    T: ProcessModuleTrait + ArchitectureTrait + VirtualReadHelperFuncs + VirtualReadUnicodeString,
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
}

impl<T> ModuleTrait for Module<T>
where
    T: ProcessModuleTrait + ArchitectureTrait + VirtualReadHelperFuncs + VirtualReadUnicodeString,
{
    // TODO: 0 should also result in an error
    fn base(&mut self) -> flow_core::Result<Address> {
        if !self.module_base.is_null() {
            return Ok(self.module_base);
        }

        let process = &mut self.process.borrow_mut();
        let proc_arch = { process.arch()? };

        self.module_base = match proc_arch.instruction_set {
            InstructionSet::X64 => addr!(process.virt_read_u64(self.peb_entry + len!(0x30))?), // self.get_offset("_LDR_DATA_TABLE_ENTRY", "DllBase")?
            InstructionSet::X86 => addr!(process.virt_read_u32(self.peb_entry + len!(0x18))?),
            _ => return Err(flow_core::Error::new("invalid process architecture")),
        };
        Ok(self.module_base)
    }

    // TODO: 0 should also result in an error
    fn size(&mut self) -> flow_core::Result<Length> {
        if !self.module_size.is_zero() {
            return Ok(self.module_size);
        }

        let process = &mut self.process.borrow_mut();
        let proc_arch = { process.arch()? };

        self.module_size = match proc_arch.instruction_set {
            InstructionSet::X64 => len!(process.virt_read_u64(self.peb_entry + len!(0x40))?), // self.get_offset("_LDR_DATA_TABLE_ENTRY", "SizeOfImage")?
            InstructionSet::X86 => len!(process.virt_read_u32(self.peb_entry + len!(0x20))?),
            _ => return Err(flow_core::Error::new("invalid process architecture")),
        };
        Ok(self.module_size)
    }

    fn name(&mut self) -> flow_core::Result<String> {
        if self.module_name != "" {
            return Ok(self.module_name.clone());
        }

        let process = &mut self.process.borrow_mut();
        let proc_arch = { process.arch()? };

        let offs = match proc_arch.instruction_set {
            InstructionSet::X64 => len!(0x58), // self.get_offset("_LDR_DATA_TABLE_ENTRY", "BaseDllName")?,
            InstructionSet::X86 => len!(0x2C),
            _ => return Err(flow_core::Error::new("invalid process architecture")),
        };

        // x64 = x64 && !wow64
        self.module_name = process.virt_read_unicode_string(self.peb_entry + offs)?;
        Ok(self.module_name.clone())
    }
}

// TODO: move exports + sections into ModuleTrait...
impl<T> Module<T>
where
    T: ProcessModuleTrait + ArchitectureTrait + VirtualReadHelper + VirtualReadHelperFuncs,
{
    // convenience wrappers (exports, sections, etc)
    pub fn export_offset(&mut self, name: &str) -> Result<Length> {
        let base = self.base()?;
        let size = self.size()?;

        let process = &mut self.process.borrow_mut();
        let buf = process.virt_read(base, size)?;
        let pe = PeView::from_bytes(&buf)?;
        match pe.get_export_by_name(name)? {
            exports::Export::Symbol(s) => Ok(Length::from(*s)),
            exports::Export::Forward(_) => Err(Error::new("Export is forwarded")),
        }
    }

    pub fn export(&mut self, name: &str) -> Result<Address> {
        Ok(self.base()? + self.export_offset(name)?)
    }

    pub fn exports(&mut self) -> Result<Vec<Export>> {
        let base = self.base()?;
        let size = self.size()?;

        let process = &mut self.process.borrow_mut();
        let buf = process.virt_read(base, size)?;
        let pe = PeView::from_bytes(&buf)?;

        Ok(pe
            .exports()?
            .by()?
            .iter_names()
            .filter(|(name, _)| name.is_ok())
            .filter(|(_, e)| e.is_ok())
            .filter(|(_, e)| e.unwrap().symbol().is_some())
            .map(|(name, e)| {
                Export::with(
                    name.unwrap().to_str().unwrap_or_default(),
                    Length::from(e.unwrap().symbol().unwrap()),
                )
            })
            .collect::<Vec<Export>>())
    }

    // TODO: port to pelite
    // section
    /*
    pub fn section(&mut self, name: &str) -> Result<Section> {
        // TODO: cache pe in this module?

        let base = self.base()?;
        let size = self.size()?;

        let process = &mut self.process.borrow_mut();
        let buf = process.virt_read(base, size)?;

        let mut pe_opts = ParseOptions::default();
        pe_opts.resolve_rva = false;
        let pe = PE::parse_with_opts(&buf, &pe_opts)?;
        Ok(pe
            .sections
            .iter()
            .filter(|s| s.real_name.clone().unwrap_or_default() == name)
            .map(Section::from)
            .nth(0)
            .ok_or_else(|| "unable to find section")?)
    }

    pub fn sections(&mut self) -> Result<Vec<Section>> {
        let base = self.base()?;
        let size = self.size()?;

        let process = &mut self.process.borrow_mut();
        let buf = process.virt_read(base, size)?;

        let mut pe_opts = ParseOptions::default();
        pe_opts.resolve_rva = false;
        let pe = PE::parse_with_opts(&buf, &pe_opts)?;
        Ok(pe
            .sections
            .iter()
            .map(Section::from)
            .collect::<Vec<Section>>())
    }*/
}

// TODO: should we really forward declare or have a back ref?
// TODO: think about forwarding dtb/typearch trait
// TODO: forward declare virtual read functions
impl<T> VirtualReadHelper for Module<T>
where
    T: ProcessModuleTrait
        + ArchitectureTrait
        + VirtualReadHelperFuncs
        + VirtualReadUnicodeString
        + VirtualReadHelper,
{
    fn virt_read(&mut self, addr: Address, len: Length) -> flow_core::Result<Vec<u8>> {
        let process = &mut self.process.borrow_mut();
        process.virt_read(addr, len)
    }
}

impl<T> VirtualWriteHelper for Module<T>
where
    T: ProcessModuleTrait
        + ArchitectureTrait
        + VirtualReadHelperFuncs
        + VirtualReadUnicodeString
        + VirtualWriteHelper,
{
    fn virt_write(&mut self, addr: Address, data: &[u8]) -> flow_core::Result<Length> {
        let process = &mut self.process.borrow_mut();
        process.virt_write(addr, data)
    }
}

/*
impl Module<T>
where T: ProcessModuleTrait + ArchitectureTrait + VirtualReadHelperFuncs + VirtualReadUnicodeString
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
