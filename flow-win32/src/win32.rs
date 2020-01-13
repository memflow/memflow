use crate::error::{Error, Result};

use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;

use flow_core::address::{Address, Length};
use flow_core::mem::*;
use flow_core::process::ProcessTrait;

use crate::kernel::StartBlock;

pub mod unicode_string;
pub use unicode_string::*;

pub mod process;
pub use process::*;
//pub mod module;
//pub use module::*;

use crate::kernel::ntos::Win32GUID;
use crate::offsets::Win32Offsets;

use pelite::{self, pe64::exports::Export, PeView};

#[derive(Debug, Clone)]
pub struct Win32 {
    pub start_block: StartBlock,

    pub kernel_base: Address,
    pub kernel_size: Length,
    pub kernel_guid: Win32GUID,

    pub eprocess_base: Address,
}

impl Win32 {
    // TODO: should this return a borrow?
    pub fn kernel_guid(&self) -> Win32GUID {
        self.kernel_guid.clone()
    }

    pub fn eprocess_list<T: VirtualRead>(
        &self,
        mem: &mut T,
        offsets: &Win32Offsets,
    ) -> Result<Vec<Address>> {
        let mut reader = VirtualReader::with(mem, self.start_block.arch, self.start_block.dtb);

        let mut eprocs = Vec::new();

        let mut eprocess = self.eprocess_base;
        loop {
            let mut next = reader.virt_read_addr(eprocess + offsets.eproc_link + offsets.blink)?;
            if next.is_null() {
                break;
            }
            next -= offsets.eproc_link;

            if next == self.eprocess_base {
                break;
            }
            eprocs.push(next);
            eprocess = next;
        }

        Ok(eprocs)
    }
}

/*
impl<T: VirtualRead> Win32<T> {
    pub fn kernel_process(&self) -> Result<KernelProcess<T>> {
        let clone = self.clone();

        let memory = &mut clone.mem.borrow_mut();

        // fetch ntoskrnl
        let header_buf =
            ntos::try_fetch_pe_header(&mut **memory, &clone.start_block, clone.kernel_base)?;
        let header = PeView::from_bytes(&header_buf)?;

        // PsActiveProcessHead
        let module_list = match header.get_export_by_name("PsLoadedModuleList")? {
            Export::Symbol(s) => clone.kernel_base + Length::from(*s),
            Export::Forward(_) => {
                return Err(Error::new(
                    "PsLoadedModuleList found but it was a forwarded export",
                ))
            }
        };

        let mut reader =
            VirtualReader::with(&mut **memory, clone.start_block.arch, self.start_block.dtb);
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
*/
