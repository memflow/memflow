use super::{page_cache::PageCache, page_cache::PageValidity, CacheValidator};
use crate::error::Result;
use crate::iter::PageChunks;
use crate::mem::phys_mem::{PhysicalMemory, PhysicalReadIterator, PhysicalWriteIterator};

use bumpalo::Bump;

pub struct CachedMemoryAccess<'a, T: PhysicalMemory, Q: CacheValidator> {
    mem: &'a mut T,
    cache: PageCache<'a, Q>,
    arena: Bump,
}

impl<'a, T: PhysicalMemory, Q: CacheValidator> CachedMemoryAccess<'a, T, Q> {
    pub fn with(mem: &'a mut T, cache: PageCache<'a, Q>) -> Self {
        Self {
            mem,
            cache,
            arena: Bump::new(),
        }
    }
}

// forward PhysicalMemory trait fncs
impl<'a, T: PhysicalMemory, Q: CacheValidator> PhysicalMemory for CachedMemoryAccess<'a, T, Q> {
    fn phys_read_iter<'b, PI: PhysicalReadIterator<'b>>(&'b mut self, iter: PI) -> Result<()> {
        self.cache.validator.update_validity();
        self.arena.reset();
        self.cache.cached_read(self.mem, iter, &self.arena)
    }

    fn phys_write_iter<'b, PI: PhysicalWriteIterator<'b>>(&'b mut self, iter: PI) -> Result<()> {
        self.cache.validator.update_validity();

        let cache = &mut self.cache;
        let mem = &mut self.mem;

        let iter = iter.inspect(move |(addr, data)| {
            if cache.is_cached_page_type(addr.page_type()) {
                for (paddr, data_chunk) in data.page_chunks(addr.address(), cache.page_size()) {
                    let mut cached_page = cache.cached_page_mut(paddr, false);
                    if let PageValidity::Valid(buf) = &mut cached_page.validity {
                        // write-back into still valid cache pages
                        let start = (paddr - cached_page.address).as_usize();
                        buf[start..(start + data_chunk.len())].copy_from_slice(data_chunk);
                    }

                    cache.put_entry(cached_page);
                }
            }
        });

        mem.phys_write_iter(iter)
    }
}
