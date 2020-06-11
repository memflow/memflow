use crate::error::Result;
use crate::pe::{pe32::MemoryPeView, MemoryPeViewContext};

use log::info;

use flow_core::mem::VirtualMemory;
use flow_core::types::{Address, Length};

use pelite::pe32::Pe;

pub fn try_get_pe_name<T: VirtualMemory + ?Sized>(
    virt_mem: &mut T,
    probe_addr: Address,
) -> Result<String> {
    let ctx = MemoryPeViewContext::new(virt_mem, probe_addr)?;
    let pe = MemoryPeView::new(&ctx)?;
    let name = pe.exports()?.dll_name()?.to_str()?;
    info!("x86::try_get_pe_name: found pe header for {}", name);
    Ok(name.to_string())
}

pub fn try_get_pe_size<T: VirtualMemory + ?Sized>(
    virt_mem: &mut T,
    probe_addr: Address,
) -> Result<Length> {
    let ctx = MemoryPeViewContext::new(virt_mem, probe_addr)?;
    let pe = MemoryPeView::new(&ctx)?;
    let size = pe.optional_header().SizeOfImage;
    info!(
        "x86::try_get_pe_size: found pe header for image with a size of {} bytes.",
        size
    );
    Ok(Length::from(size))
}
