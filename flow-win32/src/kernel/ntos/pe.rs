use crate::error::{Error, Result};

use log::{debug, info};

use pelite::{self, PeView};

use flow_core::mem::VirtualMemory;
use flow_core::types::{Address, Length};

// TODO: store pe size in windows struct so we can reference it later
pub fn probe_pe_header<T: VirtualMemory + ?Sized>(
    virt_mem: &mut T,
    probe_addr: Address,
) -> Result<String> {
    // try to probe pe header
    let pe_buf = try_fetch_pe_header(virt_mem, probe_addr)?;

    let pe = match PeView::from_bytes(&pe_buf) {
        Ok(pe) => pe,
        Err(e) => {
            debug!(
                    "probe_pe_header: potential pe header at offset {:x} could not be fully probed: {:?}",
                    probe_addr,
                    e
                );
            return Err(Error::from(e));
        }
    };

    let name = pe.exports()?.dll_name()?.to_str()?;
    info!("probe_pe_header: found pe header for {}", name);
    Ok(name.to_string())
}

pub fn try_fetch_pe_header<T: VirtualMemory + ?Sized>(
    virt_mem: &mut T,
    addr: Address,
) -> Result<Vec<u8>> {
    let size_of_image = try_fetch_pe_size(virt_mem, addr)?;
    let mut buf = vec![0; size_of_image.as_usize()];
    virt_mem.virt_read_raw_into(addr, &mut buf)?;
    Ok(buf)
}

pub fn try_fetch_pe_size<T: VirtualMemory + ?Sized>(
    virt_mem: &mut T,
    addr: Address,
) -> Result<Length> {
    // try to probe pe header
    let mut probe_buf = vec![0; Length::from_kb(4).as_usize()];
    virt_mem.virt_read_raw_into(addr, &mut probe_buf)?;

    let pe_probe = match PeView::from_bytes(&probe_buf) {
        Ok(pe) => {
            debug!("try_fetch_pe_size: found pe header.");
            pe
        }
        Err(e) => {
            debug!(
                "try_fetch_pe_size: potential pe header at offset {:x} could not be probed: {:?}",
                addr, e
            );
            return Err(Error::from(e));
        }
    };

    let opt_header = pe_probe.optional_header();
    let size_of_image = match opt_header {
        pelite::Wrap::T32(opt32) => opt32.SizeOfImage,
        pelite::Wrap::T64(opt64) => opt64.SizeOfImage,
    };
    info!(
        "try_fetch_pe_size: found pe header for image with a size of {} bytes.",
        size_of_image
    );
    Ok(Length::from(size_of_image))
}
