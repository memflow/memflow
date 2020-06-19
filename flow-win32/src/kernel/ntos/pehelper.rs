use crate::error::Result;
use crate::pe::{self, MemoryPeViewContext};

use log::info;

use flow_core::mem::VirtualMemory;
use flow_core::types::Address;

use pelite::Wrap;

pub fn try_get_pe_name<T: VirtualMemory + ?Sized>(
    virt_mem: &mut T,
    probe_addr: Address,
) -> Result<String> {
    let ctx = MemoryPeViewContext::new(virt_mem, probe_addr)?;
    let pe = pe::wrap_memory_pe_view(&ctx)?;
    let name = pe.exports()?.dll_name()?.to_str()?;
    info!("x64::try_get_pe_name: found pe header for {}", name);
    Ok(name.to_string())
}

pub fn try_get_pe_size<T: VirtualMemory + ?Sized>(
    virt_mem: &mut T,
    probe_addr: Address,
) -> Result<usize> {
    let ctx = MemoryPeViewContext::new(virt_mem, probe_addr)?;
    let pe = pe::wrap_memory_pe_view(&ctx)?;
    let size = match pe.optional_header() {
        Wrap::T32(header) => header.SizeOfImage,
        Wrap::T64(header) => header.SizeOfImage,
    };
    info!(
        "x64::try_get_pe_size: found pe header for image with a size of {} bytes.",
        size
    );
    Ok(size as usize)
}
