use super::*;
use crate::arch::Architecture;
use crate::mem::{AccessPhysicalMemory, AccessVirtualMemory};
use crate::page_chunks::PageChunksMut;
use crate::vat;

// TODO: derive virtual reads here
pub struct CachedMemoryAccess<'a, T: AccessPhysicalMemory> {
    mem: &'a mut T,
    cache: &'a mut dyn PageCache,
}

impl<'a, T: AccessPhysicalMemory> CachedMemoryAccess<'a, T> {
    pub fn with(mem: &'a mut T, cache: &'a mut dyn PageCache) -> Self {
        Self { mem, cache }
    }
}

// TODO: calling phys_read_raw_into non page alligned causes UB
// forward AccessPhysicalMemory trait fncs
impl<'a, T: AccessPhysicalMemory> AccessPhysicalMemory for CachedMemoryAccess<'a, T> {
    fn phys_read_raw_into(
        &mut self,
        addr: Address,
        page_type: PageType,
        out: &mut [u8],
    ) -> Result<()> {
        // try read from cache or fall back
        match self.cache.cached_page_type(page_type) {
            Err(_) => {
                self.mem.phys_read_raw_into(addr, page_type, out)?;
            }
            Ok(_) => {
                for (paddr, chunk) in PageChunksMut::create_from(out, addr, self.cache.page_size())
                {
                    let cached_page = self.cache.cached_page(paddr);
                    // read into page buffer and set addr
                    if !cached_page.is_valid() {
                        self.mem.phys_read_raw_into(
                            cached_page.address,
                            page_type,
                            cached_page.buf,
                        )?;
                    }

                    // copy page into out buffer
                    let start = (paddr - cached_page.address).as_usize();
                    chunk.copy_from_slice(&cached_page.buf[start..(start + chunk.len())]);

                    // update update page if it wasnt valid before
                    // this is done here due to borrowing constraints
                    let cached_addr = cached_page.address;
                    if !cached_page.is_valid() {
                        self.cache.validate_page(cached_addr, page_type);
                    }
                }
            }
        }

        Ok(())
    }

    fn phys_write_raw(&mut self, addr: Address, page_type: PageType, data: &[u8]) -> Result<()> {
        // TODO: implement writeback to cache
        self.cache.invalidate_page(addr, page_type);
        self.mem.phys_write_raw(addr, page_type, data)
    }
}

// forward AccessVirtualMemory trait fncs if memory has them implemented
impl<'a, T> AccessVirtualMemory for CachedMemoryAccess<'a, T>
where
    T: AccessPhysicalMemory + AccessVirtualMemory,
{
    fn virt_read_raw_into(
        &mut self,
        arch: Architecture,
        dtb: Address,
        addr: Address,
        out: &mut [u8],
    ) -> Result<()> {
        vat::virt_read_raw_into(self, arch, dtb, addr, out)
    }

    fn virt_write_raw(
        &mut self,
        arch: Architecture,
        dtb: Address,
        addr: Address,
        data: &[u8],
    ) -> Result<()> {
        vat::virt_write_raw(self, arch, dtb, addr, data)
    }
}
