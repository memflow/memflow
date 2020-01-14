pub mod unicode_string;
pub use unicode_string::*;

pub mod process;
pub use process::*;

pub mod module;
pub use module::*;

use crate::error::{Error, Result};
use log::info;

use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;

use flow_core::address::{Address, Length};
use flow_core::mem::*;
use flow_core::process::{OperatingSystem, ProcessTrait};

use crate::kernel::{self, ntos::Win32GUID, StartBlock};
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

// TODO:
impl OperatingSystem for Win32 {}

impl Win32 {
    pub fn try_with<T: PhysicalRead + VirtualRead>(mem: &mut T) -> Result<Self> {
        /*
        Options:
        - supply cr3 (dtb)
        - supply kernel hint
        - supply pdb
        - supply kernel offsets for basic structs (dumped windbg maybe)
        */

        // find start_block in lowstub base
        let start_block = kernel::lowstub::find(mem)?;
        info!(
            "arch={:?} va={:x} dtb={:x}",
            start_block.arch, start_block.va, start_block.dtb
        );

        // find ntoskrnl.exe base
        let (kernel_base, kernel_size) = kernel::ntos::find(mem, &start_block)?;
        info!("kernel_base={:x}", kernel_base);

        // get ntoskrnl.exe guid
        let kernel_guid = kernel::ntos::find_guid(mem, &start_block, kernel_base, kernel_size)?;
        info!("kernel_guid={:?}", kernel_guid);

        // find eprocess base
        let eprocess_base = kernel::sysproc::find(mem, &start_block, kernel_base)?;
        info!("eprocess_base={:x}", eprocess_base);

        Ok(Self {
            start_block,
            kernel_base,
            kernel_size,
            kernel_guid,
            eprocess_base,
        })
    }

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
            let mut next =
                reader.virt_read_addr(eprocess + offsets.eproc_link + offsets.list_blink)?;
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
