use std::convert::TryInto;
use std::prelude::v1::*;

use log::debug;

use memflow::error::{Error, ErrorKind, ErrorOrigin, PartialResultExt, Result};
use memflow::mem::MemoryView;
use memflow::types::{size, umem, Address};

use pelite::{self, PeView};

pub fn try_get_pe_size<T: MemoryView>(mem: &mut T, probe_addr: Address) -> Result<umem> {
    let mut probe_buf = vec![0; size::kb(4)];
    mem.read_raw_into(probe_addr, &mut probe_buf)?;

    let pe_probe = PeView::from_bytes(&probe_buf)
        .map_err(|err| Error(ErrorOrigin::OsLayer, ErrorKind::InvalidExeFile).log_trace(err))?;

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
        Ok(size_of_image as umem)
    } else {
        Err(Error(ErrorOrigin::OsLayer, ErrorKind::InvalidExeFile)
            .log_trace("pe size_of_image is zero"))
    }
}

pub fn try_get_pe_image<T: MemoryView>(mem: &mut T, probe_addr: Address) -> Result<Vec<u8>> {
    let size_of_image = try_get_pe_size(mem, probe_addr)?;
    mem.read_raw(probe_addr, size_of_image.try_into().unwrap())
        .data_part()
}

pub fn try_get_pe_name<T: MemoryView>(mem: &mut T, probe_addr: Address) -> Result<String> {
    let image = try_get_pe_image(mem, probe_addr)?;
    let pe = PeView::from_bytes(&image)
        .map_err(|err| Error(ErrorOrigin::OsLayer, ErrorKind::InvalidExeFile).log_trace(err))?;
    let name = pe
        .exports()
        .map_err(|_| {
            Error(ErrorOrigin::OsLayer, ErrorKind::InvalidExeFile)
                .log_trace("unable to get exports")
        })?
        .dll_name()
        .map_err(|_| {
            Error(ErrorOrigin::OsLayer, ErrorKind::InvalidExeFile)
                .log_trace("unable to get dll name")
        })?
        .to_str()
        .map_err(|_| {
            Error(ErrorOrigin::OsLayer, ErrorKind::Encoding)
                .log_trace("unable to convert dll name string")
        })?;
    debug!("try_get_pe_name: found pe header for {}", name);
    Ok(name.to_string())
}
