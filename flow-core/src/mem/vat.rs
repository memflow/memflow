#[cfg(test)]
mod tests;

use crate::error::Result;

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
    let aligned_len = (addr + page_size).as_page_aligned(page_size) - addr;

    if aligned_len.as_usize() >= out.len() {
        let tr = arch.virt_to_phys(mem, dtb, addr)?;
        mem.phys_read_raw_into(tr.address, tr.page.page_type, out)?;
    } else {
        let mut base = addr;

        let (mut start_buf, mut end_buf) =
            out.split_at_mut(std::cmp::min(aligned_len.as_usize(), out.len()));

        for i in [start_buf, end_buf].iter_mut() {
            for chunk in i.chunks_mut(page_size.as_usize()) {
                if let Ok(tr) = arch.virt_to_phys(mem, dtb, base) {
                    mem.phys_read_raw_into(tr.address, tr.page.page_type, chunk)?;
                }
                base += Length::from(chunk.len());
            }
        }
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
    let page_size = arch.page_size();
    let aligned_len = (addr + page_size).as_page_aligned(page_size) - addr;

    if aligned_len.as_usize() >= data.len() {
        let tr = arch.virt_to_phys(mem, dtb, addr)?;
        mem.phys_write_raw(tr.address, tr.page.page_type, data)?;
    } else {
        let mut base = addr;

        let (mut start_buf, mut end_buf) =
            data.split_at(std::cmp::min(aligned_len.as_usize(), data.len()));

        for i in [start_buf, end_buf].iter_mut() {
            for chunk in i.chunks(page_size.as_usize()) {
                if let Ok(tr) = arch.virt_to_phys(mem, dtb, base) {
                    mem.phys_write_raw(tr.address, tr.page.page_type, chunk)?;
                }
                base += Length::from(chunk.len());
            }
        }
    }

    Ok(())
}
