/*
pub mod iter;
pub use iter::*;

pub mod export;
pub use export::*;

pub mod section;
pub use section::*;
*/
use crate::error::{Error, Result};
use log::trace;

use std::cell::RefCell;
use std::rc::Rc;

use flow_core::address::{Address, Length};
use flow_core::arch::{Architecture, InstructionSet};
use flow_core::mem::*;
use flow_core::process::ModuleTrait;
use flow_core::*;

use crate::win32::{Win32, process::{Win32Process, Win32UserProcess}};
use crate::offsets::Win32Offsets;
use crate::win32::unicode_string::VirtualReadUnicodeString;

use pelite::{self, pe64::exports, PeView};

#[derive(Debug, Clone)]
pub struct Win32Module {
    peb_module: Address,
    parent_eprocess: Address, // parent "reference"

    base: Address,
    size: Length,
    name: String,

    // exports
    // sections
}

impl Win32Module {
    pub fn try_with_peb<T, U>(
        mem: &mut T,
        process: &U,
        offsets: &Win32Offsets,
        peb_module: Address,
    ) -> Result<Self>
    where
        T: VirtualRead,
        U: ProcessTrait,
    {
        let mut proc_reader = VirtualReader::with(mem, process.sys_arch(), process.dtb());

        let base = match process.proc_arch().instruction_set {
            InstructionSet::X64 => proc_reader.virt_read_addr64(peb_module + offsets.ldr_data_base_x64)?,
            InstructionSet::X86 => proc_reader.virt_read_addr32(peb_module + offsets.ldr_data_base_x86)?,
            _ => return Err(Error::new("invalid architecture")),
        };
        trace!("base={:x}", base);

        let size = match process.proc_arch().instruction_set {
            InstructionSet::X64 => Length::from(proc_reader.virt_read_u64(peb_module + offsets.ldr_data_size_x64)?),
            InstructionSet::X86 => Length::from(proc_reader.virt_read_u32(peb_module + offsets.ldr_data_size_x86)?),
            _ => return Err(Error::new("invalid architecture")),
        };
        trace!("size={:x}", size);

        let name = match process.proc_arch().instruction_set {
            InstructionSet::X64 => proc_reader.virt_read_unicode_string(peb_module + offsets.ldr_data_name_x64)?,
            InstructionSet::X86 => proc_reader.virt_read_unicode_string(peb_module + offsets.ldr_data_name_x86)?,
            _ => return Err(Error::new("invalid architecture")),
        };
        trace!("name={}", name);

        Ok(Self {
            peb_module,
            parent_eprocess: process.address(),
            base,
            size,
            name,
        })
    }

    pub fn try_with_name<T, U>(
        mem: &mut T,
        process: &U,
        offsets: &Win32Offsets,
        name: &str,
    ) -> Result<Self>
    where
        T: VirtualRead,
        U: ProcessTrait + Win32Process,
    {
        process.peb_list(mem, offsets)?
            .iter()
            .map(|peb| Win32Module::try_with_peb(mem, process, offsets, *peb))
            .filter_map(Result::ok)
            .inspect(|p| trace!("{:x} {}", p.base(), p.name()))
            .filter(|p| p.name() == name)
            .nth(0)
            .ok_or_else(|| Error::new(format!("unable to find process {}", name)))
    }
}

impl ModuleTrait for Win32Module {
    fn address(&self) -> Address {
        self.peb_module
    }

    fn parent_process(&self) -> Address {
        self.parent_eprocess
    }

    fn base(&self) -> Address {
        self.base
    }

    fn size(&self) -> Length {
        self.size
    }

    fn name(&self) -> String {
        self.name.clone()
    }
}

/*
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

        // peview.section_headers().iter().find(|sect| sect.Name == *".text")

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
*/