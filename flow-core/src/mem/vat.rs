#[cfg(test)]
mod tests;

use crate::error::Result;

use crate::address::Address;
use crate::arch::Architecture;
use crate::iter::page_chunks::{PageChunks, PageChunksMut};
use crate::mem::AccessPhysicalMemory;

#[allow(unused)]
pub fn virt_read_raw_into<T: AccessPhysicalMemory>(
    mem: &mut T,
    arch: Architecture,
    dtb: Address,
    addr: Address,
    out: &mut [u8],
) -> Result<()> {
    for (vaddr, chunk) in PageChunksMut::create_from(out, addr, arch.page_size()) {
        if let Ok(paddr) = arch.virt_to_phys(mem, dtb, vaddr) {
            mem.phys_read_raw_into(paddr, chunk)?;
        } else {
            for v in chunk.iter_mut() {
                *v = 0u8;
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
    for (vaddr, chunk) in PageChunks::create_from(data, addr, arch.page_size()) {
        if let Ok(paddr) = arch.virt_to_phys(mem, dtb, vaddr) {
            mem.phys_write_raw(paddr, chunk)?;
        }
    }

    Ok(())
}
