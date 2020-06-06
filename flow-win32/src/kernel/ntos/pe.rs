use crate::error::{Error, Result};
use crate::pe::{
    pe32::MemoryPeView as MemoryPeView32, pe64::MemoryPeView as MemoryPeView64,
    MemoryPeViewContext, PeFormat,
};

use log::info;

use flow_core::mem::VirtualMemory;
use flow_core::types::{Address, Length};

use pelite::{
    pe32::exports::Export as Export32, pe32::exports::GetProcAddress as GetProcAddress32,
    pe32::Pe as Pe32, pe64::exports::Export as Export64,
    pe64::exports::GetProcAddress as GetProcAddress64, pe64::Pe as Pe64,
};

pub fn try_get_pe_name<T: VirtualMemory + ?Sized>(
    virt_mem: &mut T,
    probe_addr: Address,
) -> Result<String> {
    let ctx = MemoryPeViewContext::new(virt_mem, probe_addr)?;
    Ok(match ctx.image_format() {
        PeFormat::Pe32 => {
            let pe = MemoryPeView32::new(&ctx)?;
            let name = pe.exports()?.dll_name()?.to_str()?;
            info!("try_get_pe_name: found pe header for {}", name);
            name
        }
        PeFormat::Pe64 => {
            let pe = MemoryPeView64::new(&ctx)?;
            let name = pe.exports()?.dll_name()?.to_str()?;
            info!("try_get_pe_name: found pe header for {}", name);
            name
        }
    }
    .to_string())
}

pub fn try_get_pe_size<T: VirtualMemory + ?Sized>(
    virt_mem: &mut T,
    probe_addr: Address,
) -> Result<Length> {
    let ctx = MemoryPeViewContext::new(virt_mem, probe_addr)?;
    Ok(Length::from(match ctx.image_format() {
        PeFormat::Pe32 => {
            let pe = MemoryPeView32::new(&ctx)?;
            let size = pe.optional_header().SizeOfImage;
            info!(
                "try_get_pe_size: found pe header for image with a size of {} bytes.",
                size
            );
            size
        }
        PeFormat::Pe64 => {
            let pe = MemoryPeView64::new(&ctx)?;
            let size = pe.optional_header().SizeOfImage;
            info!(
                "try_get_pe_size: found pe header for image with a size of {} bytes.",
                size
            );
            size
        }
    }))
}

pub fn try_get_pe_export<T: VirtualMemory + ?Sized>(
    virt_mem: &mut T,
    probe_addr: Address,
    func_name: &str,
) -> Result<Address> {
    let ctx = MemoryPeViewContext::new(virt_mem, probe_addr)?;
    Ok(match ctx.image_format() {
        PeFormat::Pe32 => {
            let pe = MemoryPeView32::new(&ctx)?;
            let proc = match pe.get_export(func_name)? {
                Export32::Symbol(s) => probe_addr + Length::from(*s),
                Export32::Forward(_) => {
                    return Err(Error::new(format!(
                        "{} found but it was a forwarded export",
                        func_name
                    )))
                }
            };
            info!(
                "try_get_pe_export: found export {} at {:x}.",
                func_name, proc
            );
            proc
        }
        PeFormat::Pe64 => {
            let pe = MemoryPeView64::new(&ctx)?;
            let proc = match pe.get_export(func_name)? {
                Export64::Symbol(s) => probe_addr + Length::from(*s),
                Export64::Forward(_) => {
                    return Err(Error::new(format!(
                        "{} found but it was a forwarded export",
                        func_name
                    )))
                }
            };
            info!(
                "try_get_pe_export: found export {} at {:x}.",
                func_name, proc
            );
            proc
        }
    })
}
