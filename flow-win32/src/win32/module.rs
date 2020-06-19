use crate::error::{Error, Result};

use log::{info, trace};

use flow_core::mem::VirtualMemory;
use flow_core::process::OsProcessModuleInfo;
use flow_core::types::{size, Address};

use pelite::{self, PeView};

#[derive(Debug, Clone)]
pub struct Win32ModuleInfo {
    pub peb_module: Address,
    pub parent_eprocess: Address, // parent "reference"

    pub base: Address,
    pub size: usize,
    pub name: String,
    // exports
    // sections
}

impl Win32ModuleInfo {
    pub fn size_of_image<T: VirtualMemory>(&self, proc_mem: &mut T) -> Result<u32> {
        let mut probe_buf = vec![0; size::kb(4)];
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
                return Err(Error::new(e));
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
        Ok(size_of_image)
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

    fn size(&self) -> usize {
        self.size
    }

    fn name(&self) -> String {
        self.name.clone()
    }
}
