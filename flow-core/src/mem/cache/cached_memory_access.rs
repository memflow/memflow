use crate::error::Result;

use super::{page_cache::PageCache, CacheValidator};

use crate::architecture::Architecture;
use crate::mem::{AccessPhysicalMemory, PhysicalReadIterator, PhysicalWriteIterator};
use crate::page_chunks::PageChunks; //, PageChunksMut};
use crate::types::{Address, Page, PhysicalAddress};
use crate::types::{Done, ToDo};
use crate::vat;

// TODO: derive virtual reads here
#[derive(VirtualAddressTranslator, AccessVirtualMemory)]
pub struct CachedMemoryAccess<'a, T: AccessPhysicalMemory, Q: CacheValidator> {
    mem: &'a mut T,
    cache: PageCache<Q>,
}

impl<'a, T: AccessPhysicalMemory, Q: CacheValidator> CachedMemoryAccess<'a, T, Q> {
    pub fn with(mem: &'a mut T, cache: PageCache<Q>) -> Self {
        Self { mem, cache }
    }
}

// forward AccessPhysicalMemory trait fncs
impl<'a, T: AccessPhysicalMemory, Q: CacheValidator> AccessPhysicalMemory
    for CachedMemoryAccess<'a, T, Q>
{
    fn phys_read_raw_iter<'b, PI: PhysicalReadIterator<'b>>(&'b mut self, iter: PI) -> Result<()> {
        self.cache.validator.update_validity();
        /*let mut rlist = smallvec::SmallVec::<[_; 64]>::new();
        self.cache.validator.update_validity();

        for &mut (addr, ref mut out) in data.iter_mut() {
            if let Some(page) = addr.page {
                // try read from cache or fall back
                if self.cache.is_cached_page_type(page.page_type) {
                    for (paddr, chunk) in
                        PageChunksMut::create_from(out, addr.address, self.cache.page_size())
                        {
                            let cached_page = self.cache.cached_page_mut(paddr);
                            // read into page buffer and set addr
                            if !cached_page.is_valid() {
                                rlist.push((cached_page.address.into(), unsafe { std::slice::from_raw_parts_mut(cached_page.buf.as_mut_ptr(), cached_page.buf.len()) }));
                                //self.mem
                                //    .phys_read_raw_into(cached_page.address.into(), cached_page.buf)?;
                            }

                            // copy page into out buffer
                            // TODO: reowkr this logic, no comptuations needed
                            let start = (paddr - cached_page.address).as_usize();
                            chunk.copy_from_slice(&cached_page.buf[start..(start + chunk.len())]);

                            // update update page if it wasnt valid before
                            // this is done here due to borrowing constraints
                            if !cached_page.is_valid() {
                                self.cache.validate_page(paddr, page.page_type);
                            }
                        }
                } else {
                    rlist.push((addr, out));
                }
            } else {
                rlist.push((addr, out));
            }
        }

        if !rlist.is_empty() {
            self.mem.phys_read_raw_iter(rlist.as_mut_slice())?;
        }

        Ok(())*/
        self.mem.phys_read_raw_iter(iter)
        //self.cache.cached_read(self.mem, iter)
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
