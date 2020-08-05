use super::{page_cache::PageCache, page_cache::PageValidity, CacheValidator};
use crate::architecture::Architecture;
use crate::error::Result;
use crate::iter::PageChunks;
use crate::mem::phys_mem::{PhysicalMemory, PhysicalReadData, PhysicalWriteData};
use crate::types::{size, PageType};

use bumpalo::Bump;

pub struct CachedMemoryAccess<'a, T: ?Sized, Q> {
    mem: &'a mut T,
    cache: PageCache<'a, Q>,
    arena: Bump,
}

impl<'a, T: PhysicalMemory + ?Sized, Q: CacheValidator> CachedMemoryAccess<'a, T, Q> {
    pub fn with(mem: &'a mut T, cache: PageCache<'a, Q>) -> Self {
        Self {
            mem,
            cache,
            arena: Bump::new(),
        }
    }

    pub fn builder(mem: &'a mut T) -> CachedMemoryAccessBuilder<'a, T, Q> {
        CachedMemoryAccessBuilder::new(mem)
    }
}

// forward PhysicalMemory trait fncs
impl<'a, T: PhysicalMemory + ?Sized, Q: CacheValidator> PhysicalMemory
    for CachedMemoryAccess<'a, T, Q>
{
    fn phys_read_raw_list(&mut self, data: &mut [PhysicalReadData]) -> Result<()> {
        self.cache.validator.update_validity();
        self.arena.reset();
        self.cache.cached_read(&mut self.mem, data, &self.arena)
    }

    fn phys_write_raw_list(&mut self, data: &[PhysicalWriteData]) -> Result<()> {
        self.cache.validator.update_validity();

        let cache = &mut self.cache;
        let mem = &mut self.mem;

        data.iter().for_each(move |(addr, data)| {
            if cache.is_cached_page_type(addr.page_type()) {
                for (paddr, data_chunk) in data.page_chunks(addr.address(), cache.page_size()) {
                    let mut cached_page = cache.cached_page_mut(paddr, false);
                    if let PageValidity::Valid(buf) = &mut cached_page.validity {
                        // write-back into still valid cache pages
                        let start = paddr - cached_page.address;
                        buf[start..(start + data_chunk.len())].copy_from_slice(data_chunk);
                    }

                    cache.put_entry(cached_page);
                }
            }
        });

        mem.phys_write_raw_list(data)
    }
}

pub struct CachedMemoryAccessBuilder<'a, T: ?Sized, Q> {
    mem: &'a mut T,
    validator: Option<Q>,
    page_size: Option<usize>,
    cache_size: usize,
    page_type_mask: PageType,
}

impl<'a, T: PhysicalMemory + ?Sized, Q: CacheValidator> CachedMemoryAccessBuilder<'a, T, Q> {
    pub fn new(mem: &'a mut T) -> Self {
        Self {
            mem,
            validator: None,
            page_size: None,
            cache_size: size::mb(2),
            page_type_mask: PageType::PAGE_TABLE | PageType::READ_ONLY,
        }
    }

    pub fn build(self) -> Result<CachedMemoryAccess<'a, T, Q>> {
        Ok(CachedMemoryAccess::with(
            self.mem,
            PageCache::with_page_size(
                self.page_size.ok_or("page_size must be initialized")?,
                self.cache_size,
                self.page_type_mask,
                self.validator.ok_or("validator must be initialized")?,
            ),
        ))
    }

    pub fn validator(mut self, validator: Q) -> Self {
        self.validator = Some(validator);
        self
    }

    pub fn page_size(mut self, page_size: usize) -> Self {
        self.page_size = Some(page_size);
        self
    }

    pub fn cache_size(mut self, cache_size: usize) -> Self {
        self.cache_size = cache_size;
        self
    }

    pub fn arch(mut self, arch: Architecture) -> Self {
        self.page_size = Some(arch.page_size());
        self
    }

    pub fn page_type_mask(mut self, page_type_mask: PageType) -> Self {
        self.page_type_mask = page_type_mask;
        self
    }
}
