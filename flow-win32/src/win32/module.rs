use crate::error::{Error, Result};
use log::trace;

use flow_core::mem::VirtualMemory;
use flow_core::process::OsProcessModuleInfo;

use crate::win32::*;

use pelite::{self, PeView};

#[derive(Debug, Clone)]
pub struct Win32ModuleInfo {
    pub peb_module: Address,
    pub parent_eprocess: Address, // parent "reference"

    pub base: Address,
    pub size: Length,
    pub name: String,
    // exports
    // sections
}

impl Win32ModuleInfo {
    // read_image() - reads the entire image into memory
    pub fn read_image<T: VirtualMemory>(
        &self,
        proc_mem: &mut T,
        process: &Win32ProcessInfo,
    ) -> Result<Vec<u8>> {
        let mut probe_buf = vec![0; Length::from_kb(4).as_usize()];
        proc_mem.virt_read_raw_into(self.base, &mut probe_buf)?;

        let pe_probe = match PeView::from_bytes(&probe_buf) {
            Ok(pe) => {
                trace!("found pe header.");
                pe
            }
            Err(e) => {
                trace!(
                    "pe header at offset {:x} could not be probed: {:?}",
                    self.base,
                    e
                );
                return Err(Error::from(e));
            }
        };

        let opt_header = pe_probe.optional_header();
        let size_of_image = match opt_header {
            pelite::Wrap::T32(opt32) => opt32.SizeOfImage,
            pelite::Wrap::T64(opt64) => opt64.SizeOfImage,
        };
        if size_of_image == 0 {
            return Err(Error::new("unable to read size_of_image"));
        }
        info!("found pe header with a size of {} bytes.", size_of_image);

        let mut buf = vec![0; size_of_image as usize];
        proc_mem.virt_read_raw_into(self.base, &mut buf)?;
        Ok(buf)
    }
}

impl OsProcessModuleInfo for Win32ModuleInfo {
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
