#[cfg(test)]
mod tests;

use crate::error::{Error, Result};

use log::trace;

use crate::address::{Address, Length};
use crate::arch::Architecture;
use crate::mem::AccessPhysicalMemory;

#[allow(unused)]
pub fn virt_read_raw_into<T: AccessPhysicalMemory>(
    mem: &mut T,
    arch: Architecture,
    dtb: Address,
    addr: Address,
    out: &mut [u8],
) -> Result<()> {
    let page_size = arch.page_size();
    let mut base = addr;
    let end = base + Length::from(out.len());

    while base < end {
        let mut aligned_len = (base + page_size).as_page_aligned(page_size) - base;
        if base + aligned_len > end {
            aligned_len = end - base;
        }

        if let Ok((pa, pt)) = arch.vtop(mem, dtb, base) {
            let offset = (base - addr).as_usize();
            mem.phys_read_raw_into(pa, pt, &mut out[offset..(offset + aligned_len.as_usize())])?;
        } else {
            // skip
            trace!("pa is null, skipping page");
        }

        base += aligned_len;
    }
    Ok(())
}

#[allow(unused)]
pub fn virt_write_raw<T: AccessPhysicalMemory>(
    mem: &mut T,
    arch: Architecture,
    dtb: Address,
    addr: Address,
    data: &[u8],
) -> Result<()> {
    let (pa, pt) = arch.vtop(mem, dtb, addr)?;
    if pa.is_null() {
        // TODO: add more debug info
        Err(Error::new(
            "virt_write(): unable to resolve physical address",
        ))
    } else {
        mem.phys_write_raw(pa, pt, data)
    }
}
