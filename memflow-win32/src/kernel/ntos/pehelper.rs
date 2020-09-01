use std::prelude::v1::*;

use crate::error::{Error, Result};

use log::{debug, info};

use memflow::error::PartialResultExt;
use memflow::mem::VirtualMemory;
use memflow::types::{size, Address};

use pelite::{self, PeView};

pub fn try_get_pe_size<T: VirtualMemory>(virt_mem: &mut T, probe_addr: Address) -> Result<usize> {
    let mut probe_buf = vec![0; size::kb(4)];
    virt_mem.virt_read_raw_into(probe_addr, &mut probe_buf)?;

    let pe_probe = PeView::from_bytes(&probe_buf).map_err(Error::PE)?;

    let opt_header = pe_probe.optional_header();
    let size_of_image = match opt_header {
        pelite::Wrap::T32(opt32) => opt32.SizeOfImage,
        pelite::Wrap::T64(opt64) => opt64.SizeOfImage,
    };
    if size_of_image > 0 {
        debug!(
            "found pe header for image with a size of {} bytes.",
            size_of_image
        );
        Ok(size_of_image as usize)
    } else {
        Err(Error::Initialization("pe size_of_image is zero"))
    }
}

pub fn try_get_pe_image<T: VirtualMemory>(
    virt_mem: &mut T,
    probe_addr: Address,
) -> Result<Vec<u8>> {
    let size_of_image = try_get_pe_size(virt_mem, probe_addr)?;
    virt_mem
        .virt_read_raw(probe_addr, size_of_image)
        .data_part()
        .map_err(Error::Core)
}

pub fn try_get_pe_name<T: VirtualMemory>(virt_mem: &mut T, probe_addr: Address) -> Result<String> {
    let image = try_get_pe_image(virt_mem, probe_addr)?;
    let pe = PeView::from_bytes(&image).map_err(Error::PE)?;
    let name = pe
        .exports()
        .map_err(|_| Error::Initialization("unable to get exports"))?
        .dll_name()
        .map_err(|_| Error::Initialization("unable to get dll name"))?
        .to_str()?;
    info!("try_get_pe_name: found pe header for {}", name);
    Ok(name.to_string())
}
