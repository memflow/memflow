use crate::error::Result;

use super::{page_cache::PageCache, CacheValidator};

use crate::architecture::Architecture;
use crate::mem::{AccessPhysicalMemoryRaw, PhysicalReadIterator, PhysicalWriteIterator};
use crate::page_chunks::PageChunks;
use crate::types::{Address, Page, PhysicalAddress};
use crate::vat;
use bumpalo::Bump;

#[derive(VirtualAddressTranslatorRaw, AccessVirtualMemoryRaw)]
pub struct CachedMemoryAccess<'a, T: AccessPhysicalMemoryRaw, Q: CacheValidator> {
    mem: &'a mut T,
    cache: PageCache<Q>,
    arena: Bump,
}

impl<'a, T: AccessPhysicalMemoryRaw, Q: CacheValidator> CachedMemoryAccess<'a, T, Q> {
    pub fn with(mem: &'a mut T, cache: PageCache<Q>) -> Self {
        Self {
            mem,
            cache,
            arena: Bump::new(),
        }
    }
}

// forward AccessPhysicalMemoryRaw trait fncs
impl<'a, T: AccessPhysicalMemoryRaw, Q: CacheValidator> AccessPhysicalMemoryRaw
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
    ) -> Result<()> {
        self.cache.validator.update_validity();

        let cache = &mut self.cache;
        let mem = &mut self.mem;

        let iter = iter.inspect(move |(addr, data)| {
            if cache.is_cached_page_type(addr.page_type()) {
                for (paddr, data_chunk) in
                    PageChunks::create_from(data, addr.address(), cache.page_size())
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
        });

        mem.phys_write_raw_iter(iter)
    }
}
