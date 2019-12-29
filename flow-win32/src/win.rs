use crate::error::{Error, Result};

use log::info;

use std::cell::RefCell;
use std::rc::Rc;

use flow_core::address::{Address, Length};
use flow_core::mem::*;
use flow_core::process::ProcessTrait;

use crate::kernel::StartBlock;

pub mod types;

pub mod unicode_string;
pub use unicode_string::*;

pub mod module;
pub use module::{Module, ModuleIterator};
pub mod process;
use crate::win::process::ProcessModuleHelper;
pub use process::{KernelProcess, ProcessIterator, ProcessModuleTrait, UserProcess};

use goblin::pe::options::ParseOptions;
use goblin::pe::PE;

// TODO: cache processes somewhat?
pub struct Windows<T: VirtualRead> {
    pub mem: Rc<RefCell<T>>,

    pub start_block: StartBlock,
    pub kernel_base: Address,
    pub eprocess_base: Address,

    pub kernel_pdb: Option<Rc<RefCell<types::PDB>>>,
}

impl<T: VirtualRead> Clone for Windows<T>
where
    Rc<RefCell<T>>: Clone,
    StartBlock: Clone,
    Address: Clone,
    Option<types::PDB>: Clone,
{
    fn clone(&self) -> Self {
        Self {
            mem: self.mem.clone(),
            start_block: self.start_block,
            kernel_base: self.kernel_base,
            eprocess_base: self.eprocess_base,
            kernel_pdb: self.kernel_pdb.clone(),
        }
    }
}

impl<T: VirtualRead> Windows<T> {
    pub fn kernel_process(&self) -> Result<KernelProcess<T>> {
        let clone = self.clone();

        let memory = &mut clone.mem.borrow_mut();
        let mut reader =
            VirtualReader::with(&mut **memory, clone.start_block.arch, self.start_block.dtb);
        let header_buf = reader.virt_read(clone.kernel_base, Length::from_mb(32))?;

        let mut pe_opts = ParseOptions::default();
        pe_opts.resolve_rva = false;

        let header = PE::parse_with_opts(&header_buf, &pe_opts).unwrap(); // TODO: error
        let module_list = header
            .exports
            .iter()
            .filter(|e| e.name.unwrap_or_default() == "PsLoadedModuleList") // PsActiveProcessHead
            .inspect(|e| info!("found eat entry: {:?}", e))
            .nth(0)
            .ok_or_else(|| Error::new("unable to find export PsLoadedModuleList"))
            .and_then(|e| Ok(clone.kernel_base + Length::from(e.rva)))?;

        let addr = reader.virt_read_addr(module_list)?;
        let rc = Rc::new(RefCell::new(self.clone()));
        Ok(KernelProcess::with(rc, addr))
    }

    pub fn process_iter(&self) -> ProcessIterator<T> {
        let rc = Rc::new(RefCell::new(self.clone()));
        ProcessIterator::new(rc)
    }

    // TODO: check if first module matches process name / alive check?
    pub fn process(&self, name: &str) -> Result<UserProcess<T>> {
        Ok(self
            .process_iter()
            .filter_map(|mut m| {
                if m.name().unwrap_or_default() == name {
                    Some(m)
                } else {
                    None
                }
            })
            .filter(|m| m.first_module().is_ok())
            .nth(0)
            .ok_or_else(|| "unable to find process")?)
    }
}
