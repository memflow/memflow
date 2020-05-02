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
    let aligned_len = std::cmp::min(
        (addr + page_size).as_page_aligned(page_size) - addr,
        Length::from(out.len()),
    );

    let (mut start_buf, mut end_buf) = out.split_at_mut(aligned_len.as_usize());

    let mut base = addr;

    let mut thing = |buf: &mut [u8]| -> Result<()> {
        if let Ok((pa, pt)) = arch.vtop(mem, dtb, base) {
            mem.phys_read_raw_into(pa, pt, buf)?;
        }
        base += Length::from(buf.len());
        Ok(())
    };

    thing(start_buf)?;
    end_buf
        .chunks_mut(page_size.as_usize())
        .try_for_each(thing)?;

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
