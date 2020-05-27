use crate::error::Result;

use super::{page_cache::PageCache, CacheValidator};

use crate::architecture::Architecture;
use crate::mem::{AccessPhysicalMemory, PhysicalReadIterator, PhysicalWriteIterator};
use crate::page_chunks::PageChunks; //, PageChunksMut};
use crate::types::{Address, Page, PhysicalAddress};
use crate::types::{Done, ToDo};
use crate::vat;
use bumpalo::Bump;

// TODO: derive virtual reads here
#[derive(VirtualAddressTranslator, AccessVirtualMemory)]
pub struct CachedMemoryAccess<'a, T: AccessPhysicalMemory, Q: CacheValidator> {
    mem: &'a mut T,
    cache: PageCache<Q>,
    arena: Bump,
}

impl<'a, T: AccessPhysicalMemory, Q: CacheValidator> CachedMemoryAccess<'a, T, Q> {
    pub fn with(mem: &'a mut T, cache: PageCache<Q>) -> Self {
        Self {
            mem,
            cache,
            arena: Bump::new(),
        }
    }
}

// forward AccessPhysicalMemory trait fncs
impl<'a, T: AccessPhysicalMemory, Q: CacheValidator> AccessPhysicalMemory
    for CachedMemoryAccess<'a, T, Q>
{
    fn phys_read_raw_iter<'b, PI: PhysicalReadIterator<'b>>(&'b mut self, iter: PI) -> Result<()> {
        self.cache.validator.update_validity();
        self.arena.reset();
        self.cache.cached_read(self.mem, iter, &self.arena)
    }

    fn phys_write_raw_iter<'b, PI: PhysicalWriteIterator<'b>>(
        &'b mut self,
        iter: PI,
    ) -> Box<dyn PhysicalWriteIterator<'b>> {
        self.cache.validator.update_validity();

        let cache = &mut self.cache;
        let mem = &mut self.mem;

        let iter = iter.inspect(move |x| {
            if let ToDo((addr, data)) = x {
                if let Some(page) = addr.page {
                    if cache.is_cached_page_type(page.page_type) {
                        for (paddr, data_chunk) in
                            PageChunks::create_from(data, addr.address, cache.page_size())
                        {
                            let cached_page = cache.cached_page_mut(paddr);
                            if cached_page.is_valid() {
                                // write-back into still valid cache pages
                                let start = (paddr - cached_page.address).as_usize();
                                cached_page.buf[start..(start + data_chunk.len())]
                                    .copy_from_slice(data_chunk);
                            }
                        }
                    }
                }
            }
        });

        mem.phys_write_raw_iter(iter)
    }
}

// forward AccessVirtualMemory trait fncs if memory has them implemented
/*impl<'a, T, Q> AccessVirtualMemory for CachedMemoryAccess<'a, T, Q>
where
    T: AccessPhysicalMemory,
    Q: CacheValidator,
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

    fn virt_page_info(&mut self, arch: Architecture, dtb: Address, addr: Address) -> Result<Page> {
        vat::virt_page_info(self, arch, dtb, addr)
    }
}*/
