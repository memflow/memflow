use super::{CacheValidator, PageType};
use crate::architecture::Architecture;
use crate::error::Error;
use crate::iter::{FlowIters, PageChunks};
use crate::mem::phys_mem::{PhysicalMemory, PhysicalReadData, PhysicalReadIterator};
use crate::types::{Address, Length, PhysicalAddress};
use bumpalo::{collections::Vec as BumpVec, Bump};
use std::alloc::{alloc_zeroed, Layout};

pub struct CacheEntry<'a> {
    pub valid: bool,
    pub address: Address,
    pub should_validate: bool,
    pub buf: &'a mut [u8],
}

impl<'a> CacheEntry<'a> {
    pub fn with(valid: bool, should_validate: bool, address: Address, buf: &'a mut [u8]) -> Self {
        Self {
            valid,
            should_validate,
            address,
            buf,
        }
    }

    pub fn is_valid(&self) -> bool {
        self.valid
    }

    pub fn address(&self) -> Address {
        self.address
    }

    pub fn should_validate(&self) -> bool {
        self.should_validate
    }
}

#[derive(Clone)]
pub struct PageCache<T: CacheValidator> {
    address: Box<[Address]>,
    cache: Box<[u8]>,
    address_once_validated: Box<[Address]>,
    page_size: Length,
    page_type_mask: PageType,
    pub validator: T,
}

impl<T: CacheValidator> PageCache<T> {
    pub fn new(arch: Architecture, size: Length, page_type_mask: PageType, validator: T) -> Self {
        Self::with_page_size(arch.page_size(), size, page_type_mask, validator)
    }

    pub fn with_page_size(
        page_size: Length,
        size: Length,
        page_type_mask: PageType,
        mut validator: T,
    ) -> Self {
        let cache_entries = size.as_usize() / page_size.as_usize();

        let layout =
            Layout::from_size_align(cache_entries * page_size.as_usize(), page_size.as_usize())
                .unwrap();

        let cache = unsafe {
            Box::from_raw(std::slice::from_raw_parts_mut(
                alloc_zeroed(layout),
                layout.size(),
            ))
        };

        validator.allocate_slots(cache_entries);

        Self {
            address: vec![Address::INVALID; cache_entries].into_boxed_slice(),
            cache,
            address_once_validated: vec![Address::INVALID; cache_entries].into_boxed_slice(),
            page_size,
            page_type_mask,
            validator,
        }
    }

    fn page_index(&self, addr: Address) -> usize {
        (addr.as_page_aligned(self.page_size).as_usize() / self.page_size.as_usize())
            % self.address.len()
    }

    fn page_and_info_from_index(&mut self, idx: usize) -> (&mut [u8], &mut Address, &mut Address) {
        let start = self.page_size.as_usize() * idx;
        (
            &mut self.cache[start..(start + self.page_size.as_usize())],
            &mut self.address[idx],
            &mut self.address_once_validated[idx],
        )
    }

    fn page_from_index(&mut self, idx: usize) -> &mut [u8] {
        let start = self.page_size.as_usize() * idx;
        &mut self.cache[start..(start + self.page_size.as_usize())]
    }

    fn try_page(
        &mut self,
        addr: Address,
    ) -> std::result::Result<&mut [u8], (&mut [u8], &mut Address, &mut Address)> {
        let page_index = self.page_index(addr);
        if self.address[page_index] == addr.as_page_aligned(self.page_size)
            && self.validator.is_slot_valid(page_index)
        {
            Ok(self.page_from_index(page_index))
        } else {
            Err(self.page_and_info_from_index(page_index))
        }
    }

    pub fn page_size(&self) -> Length {
        self.page_size
    }

    pub fn is_cached_page_type(&self, page_type: PageType) -> bool {
        self.page_type_mask.contains(page_type)
    }

    pub fn cached_page_mut(&mut self, addr: Address) -> CacheEntry {
        let page_size = self.page_size;
        let aligned_addr = addr.as_page_aligned(page_size);
        match self.try_page(addr) {
            Ok(page) => CacheEntry {
                valid: true,
                should_validate: false,
                address: aligned_addr,
                buf: page,
            },
            Err((page, _, addr_once_validated)) => {
                if *addr_once_validated == Address::INVALID {
                    *addr_once_validated = aligned_addr;
                }
                CacheEntry {
                    valid: false,
                    should_validate: aligned_addr == *addr_once_validated,
                    address: aligned_addr,
                    buf: page,
                }
            }
        }
    }

    pub fn validate_page(&mut self, addr: Address, page_type: PageType) {
        if self.page_type_mask.contains(page_type) {
            let idx = self.page_index(addr);
            let aligned_addr = addr.as_page_aligned(self.page_size);
            let page_info = self.page_and_info_from_index(idx);
            *page_info.1 = aligned_addr;
            self.validator.validate_slot(idx);
            debug_assert_eq!(self.address_once_validated[idx], aligned_addr);
            self.address_once_validated[idx] = Address::INVALID;
        }
    }

    pub fn invalidate_page(&mut self, addr: Address, page_type: PageType) {
        if self.page_type_mask.contains(page_type) {
            let idx = self.page_index(addr);
            let page_info = self.page_and_info_from_index(idx);
            *page_info.1 = Address::null();
            self.validator.invalidate_slot(idx);
            self.address_once_validated[idx] = Address::INVALID;
        }
    }

    fn cached_read_single<F: PhysicalMemory>(
        &mut self,
        mem: &mut F,
        addr: PhysicalAddress,
        out: &mut [u8],
    ) -> Result<(), Error> {
        // try read from cache or fall back
        if self.is_cached_page_type(addr.page_type()) {
            for (paddr, chunk) in out.page_chunks(addr.address(), self.page_size()) {
                let cached_page = self.cached_page_mut(paddr);

                if cached_page.should_validate() {
                    mem.phys_read_raw_into(cached_page.address.into(), cached_page.buf)?;
                }

                if cached_page.is_valid() || cached_page.should_validate() {
                    let start = (paddr - cached_page.address).as_usize();
                    chunk.copy_from_slice(&cached_page.buf[start..(start + chunk.len())]);
                }

                if cached_page.should_validate() {
                    self.validate_page(paddr, addr.page_type());
                }
            }
        } else {
            mem.phys_read_raw_into(addr, out)?;
        }
        Ok(())
    }

    pub fn split_to_chunks(
        (addr, out): PhysicalReadData<'_>,
        page_size: Length,
    ) -> impl PhysicalReadIterator<'_> {
        out.page_chunks(addr.address(), page_size)
            .map(move |(paddr, chunk)| {
                (
                    PhysicalAddress::with_page(paddr, addr.page_type(), addr.page_size()),
                    chunk,
                )
            })
    }

    fn item_overlaps_pages(&self, elem: &Option<PhysicalReadData<'_>>) -> bool {
        if let Some((addr, data)) = elem {
            addr.address() + Length::from(data.len()) - addr.page_base() > self.page_size
        } else {
            false
        }
    }

    #[allow(clippy::never_loop)]
    pub fn cached_read<'a, F: PhysicalMemory, PI: PhysicalReadIterator<'a>>(
        &'a mut self,
        mem: &'a mut F,
        iter: PI,
        arena: &Bump,
    ) -> Result<(), crate::Error> {
        let page_size = self.page_size;

        let mut iter = iter.double_peekable();

        if iter.is_next_last() && !self.item_overlaps_pages(iter.double_peek().0) {
            if let Some((addr, ref mut out)) = iter.next() {
                self.cached_read_single(mem, addr, out)
            } else {
                Ok(())
            }
        } else {
            let mut next = iter.next();
            let mut clist = BumpVec::new_in(arena);
            let mut wlist = BumpVec::new_in(arena);

            while let Some((addr, out)) = next {
                out.page_chunks(addr.address(), page_size)
                    .for_each(|(paddr, chunk)| {
                        let (addr, out) = (
                            PhysicalAddress::with_page(paddr, addr.page_type(), addr.page_size()),
                            chunk,
                        );

                        loop {
                            if self.is_cached_page_type(addr.page_type()) {
                                let cached_page = self.cached_page_mut(addr.address());

                                if cached_page.is_valid() {
                                    clist.push((addr, out));
                                    break;
                                } else if cached_page.should_validate() {
                                    //TODO: This has to become safe somehow
                                    let buf = unsafe {
                                        std::slice::from_raw_parts_mut(
                                            cached_page.buf.as_mut_ptr(),
                                            cached_page.buf.len(),
                                        )
                                    };

                                    wlist.push((PhysicalAddress::from(cached_page.address), buf));
                                    clist.push((addr, out));
                                    self.validate_page(addr.address(), addr.page_type());
                                    break;
                                }
                            }

                            wlist.push((addr, out));
                            break;
                        }
                    });

                next = iter.next();

                if next.is_none() || wlist.len() >= 64 || clist.len() >= 64 {
                    if !wlist.is_empty() {
                        mem.phys_read_iter(wlist.into_iter())?;
                        wlist = BumpVec::new_in(arena);
                    }

                    while let Some((addr, out)) = clist.pop() {
                        let paddr = addr.address();
                        let aligned_addr = paddr.as_page_aligned(self.page_size);
                        let cached_page = self.page_from_index(self.page_index(paddr));
                        let start = (paddr - aligned_addr).as_usize();
                        let cached_page =
                            cached_page.split_at_mut(start).1.split_at_mut(out.len()).0;
                        out.copy_from_slice(cached_page);
                    }
                }
            }

            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::architecture::Architecture;
    use crate::dummy::DummyMemory;
    use crate::mem::cache::page_cache::PageCache;
    use crate::mem::cache::timed_validator::TimedCacheValidator;
    use crate::mem::{VirtualFromPhysical, VirtualMemory};
    use crate::types::{Address, Length, PhysicalAddress};
    use crate::*;
    use rand::{thread_rng, Rng};

    fn diff_regions<'a>(
        mut r1: &'a [u8],
        mut r2: &'a [u8],
        diff_size: usize,
    ) -> Vec<(usize, &'a [u8], &'a [u8])> {
        let mut diffs = vec![];

        assert!(r1.len() == r2.len());

        let mut cidx = 0;

        while !r1.is_empty() {
            let splitc = core::cmp::min(r1.len(), diff_size);
            let (r1l, r1r) = r1.split_at(splitc);
            let (r2l, r2r) = r2.split_at(splitc);
            r1 = r1r;
            r2 = r2r;

            if r1l != r2l {
                diffs.push((cidx, r1l, r2l));
            }

            cidx += splitc;
        }

        diffs
    }

    /// Test cached memory read both with a random seed and a predetermined one.
    ///
    /// The predetermined seed was found to be problematic when it comes to memory overlap
    #[test]
    fn big_virt_buf() {
        for &seed in &[0x3ffd235c5194dedf, thread_rng().gen_range(0, !0u64)] {
            let mut dummy_mem = DummyMemory::with_seed(Length::from_mb(512), seed);

            let virt_size = Length::from_mb(18);
            let mut test_buf = vec![0_u64; virt_size.as_usize() / 8];

            for i in &mut test_buf {
                *i = thread_rng().gen::<u64>();
            }

            let test_buf = unsafe {
                std::slice::from_raw_parts(test_buf.as_ptr() as *const u8, virt_size.as_usize())
            };

            let (dtb, virt_base) = dummy_mem.alloc_dtb(virt_size, &test_buf);
            let arch = Architecture::X64;
            println!("dtb={:x} virt_base={:x} seed={:x}", dtb, virt_base, seed);

            let mut buf_nocache = vec![0_u8; test_buf.len()];
            {
                let mut virt_mem = VirtualFromPhysical::new(&mut dummy_mem, arch, arch, dtb);
                virt_mem
                    .virt_read_raw_into(virt_base, buf_nocache.as_mut_slice())
                    .unwrap();
            }

            assert!(
                buf_nocache == test_buf,
                "buf_nocache ({:?}..{:?}) != test_buf ({:?}..{:?})",
                &buf_nocache[..16],
                &buf_nocache[buf_nocache.len() - 16..],
                &test_buf[..16],
                &test_buf[test_buf.len() - 16..]
            );

            let cache = PageCache::new(
                arch,
                Length::from_mb(2),
                PageType::PAGE_TABLE | PageType::READ_ONLY,
                TimedCacheValidator::new(coarsetime::Duration::from_secs(100)),
            );
            let mut mem_cache = CachedMemoryAccess::with(&mut dummy_mem, cache);
            let mut buf_cache = vec![0_u8; buf_nocache.len()];
            {
                let mut virt_mem = VirtualFromPhysical::new(&mut mem_cache, arch, arch, dtb);
                virt_mem
                    .virt_read_raw_into(virt_base, buf_cache.as_mut_slice())
                    .unwrap();
            }

            assert!(
                buf_nocache == buf_cache,
                "buf_nocache\n({:?}..{:?}) != buf_cache\n({:?}..{:?})\nDiff:\n{:?}",
                &buf_nocache[..16],
                &buf_nocache[buf_nocache.len() - 16..],
                &buf_cache[..16],
                &buf_cache[buf_cache.len() - 16..],
                diff_regions(&buf_nocache, &buf_cache, 32)
            );
        }
    }

    #[test]
    fn cache_invalidity_cached() {
        let mut dummy_mem = DummyMemory::new(Length::from_mb(64));
        let mem_ptr = &mut dummy_mem as *mut DummyMemory;
        let virt_size = Length::from_mb(8);
        let mut buf_start = vec![0_u8; 64];
        for (i, item) in buf_start.iter_mut().enumerate() {
            *item = (i % 256) as u8;
        }
        let (dtb, virt_base) = dummy_mem.alloc_dtb(virt_size, &buf_start);
        let arch = Architecture::X64;

        let cache = PageCache::new(
            arch,
            Length::from_mb(2),
            PageType::PAGE_TABLE | PageType::READ_ONLY | PageType::WRITEABLE,
            TimedCacheValidator::new(coarsetime::Duration::from_secs(100)),
        );

        let mut mem_cache = CachedMemoryAccess::with(&mut dummy_mem, cache);

        //Modifying the memory from other channels should leave the cached page unchanged
        let mut cached_buf = vec![0_u8; 64];
        {
            let mut virt_mem = VirtualFromPhysical::new(&mut mem_cache, arch, arch, dtb);
            virt_mem
                .virt_read_raw_into(virt_base, cached_buf.as_mut_slice())
                .unwrap();
        }

        let mut write_buf = cached_buf.clone();
        write_buf[16..20].copy_from_slice(&[255, 255, 255, 255]);
        {
            let mut virt_mem =
                VirtualFromPhysical::new(unsafe { mem_ptr.as_mut().unwrap() }, arch, arch, dtb);
            virt_mem
                .virt_write_raw(virt_base, write_buf.as_slice())
                .unwrap();
        }

        let mut check_buf = vec![0_u8; 64];
        {
            let mut virt_mem = VirtualFromPhysical::new(&mut mem_cache, arch, arch, dtb);
            virt_mem
                .virt_read_raw_into(virt_base, check_buf.as_mut_slice())
                .unwrap();
        }

        assert_eq!(cached_buf, check_buf);
        assert_ne!(check_buf, write_buf);
    }

    #[test]
    fn cache_invalidity_non_cached() {
        let mut dummy_mem = DummyMemory::new(Length::from_mb(64));
        let mem_ptr = &mut dummy_mem as *mut DummyMemory;
        let virt_size = Length::from_mb(8);
        let mut buf_start = vec![0_u8; 64];
        for (i, item) in buf_start.iter_mut().enumerate() {
            *item = (i % 256) as u8;
        }
        let (dtb, virt_base) = dummy_mem.alloc_dtb(virt_size, &buf_start);
        let arch = Architecture::X64;

        //alloc_dtb creates a page table with all writeable pages, we disable cache for them
        let cache = PageCache::new(
            arch,
            Length::from_mb(2),
            PageType::PAGE_TABLE | PageType::READ_ONLY,
            TimedCacheValidator::new(coarsetime::Duration::from_secs(100)),
        );

        let mut mem_cache = CachedMemoryAccess::with(&mut dummy_mem, cache);

        //Modifying the memory from other channels should leave the cached page unchanged
        let mut cached_buf = vec![0_u8; 64];
        {
            let mut virt_mem = VirtualFromPhysical::new(&mut mem_cache, arch, arch, dtb);
            virt_mem
                .virt_read_raw_into(virt_base, cached_buf.as_mut_slice())
                .unwrap();
        }

        let mut write_buf = cached_buf.clone();
        write_buf[16..20].copy_from_slice(&[255, 255, 255, 255]);
        {
            let mut virt_mem =
                VirtualFromPhysical::new(unsafe { mem_ptr.as_mut().unwrap() }, arch, arch, dtb);
            virt_mem
                .virt_write_raw(virt_base, write_buf.as_slice())
                .unwrap();
        }

        let mut check_buf = vec![0_u8; 64];
        {
            let mut virt_mem = VirtualFromPhysical::new(mem_cache, arch, arch, dtb);
            virt_mem
                .virt_read_raw_into(virt_base, check_buf.as_mut_slice())
                .unwrap();
        }

        assert_ne!(cached_buf, check_buf);
        assert_eq!(check_buf, write_buf);
    }

    /// Test overlap of page cache.
    ///
    /// This test will fail if the page marks a memory region for copying from the cache, but also
    /// caches a different page in the entry before the said copy is operation is made.
    #[test]
    fn cache_phys_mem_overlap() {
        let mut dummy_mem = DummyMemory::new(Length::from_mb(16));

        let buf_size = Length::from_kb(8);
        let mut buf_start = vec![0_u8; buf_size.as_usize()];
        for (i, item) in buf_start.iter_mut().enumerate() {
            *item = ((i / 115) % 256) as u8;
        }

        let address = Address::from(0);

        let addr =
            PhysicalAddress::with_page(address, PageType::from_writeable_bit(false), 0x1000.into());

        dummy_mem
            .phys_write_raw(addr, buf_start.as_slice())
            .unwrap();

        let arch = Architecture::X64;

        let cache = PageCache::new(
            arch,
            Length::from_kb(4),
            PageType::PAGE_TABLE | PageType::READ_ONLY,
            TimedCacheValidator::new(coarsetime::Duration::from_secs(100)),
        );

        let mut mem_cache = CachedMemoryAccess::with(&mut dummy_mem, cache);

        let mut buf_1 = vec![0_u8; buf_size.as_usize()];
        mem_cache
            .phys_read_into(addr, buf_1.as_mut_slice())
            .unwrap();

        assert!(
            buf_start == buf_1,
            "buf_start != buf_1; diff: {:?}",
            diff_regions(&buf_start, &buf_1, 128)
        );

        let addr = PhysicalAddress::with_page(
            address + Length::from_kb(4),
            PageType::from_writeable_bit(false),
            0x1000.into(),
        );

        let mut buf_2 = vec![0_u8; buf_size.as_usize()];
        mem_cache
            .phys_read_into(addr, buf_2.as_mut_slice())
            .unwrap();

        assert!(
            buf_1[0x1000..] == buf_2[..0x1000],
            "buf_1 != buf_2; diff: {:?}",
            diff_regions(&buf_1[0x1000..], &buf_2[..0x1000], 128)
        );
    }

    #[test]
    fn cache_phys_mem() {
        let mut dummy_mem = DummyMemory::new(Length::from_mb(16));

        let mut buf_start = vec![0_u8; 64];
        for (i, item) in buf_start.iter_mut().enumerate() {
            *item = (i % 256) as u8;
        }

        let address = Address::from(0x5323);

        let addr =
            PhysicalAddress::with_page(address, PageType::from_writeable_bit(false), 0x1000.into());

        dummy_mem
            .phys_write_raw(addr, buf_start.as_slice())
            .unwrap();

        let arch = Architecture::X64;

        let cache = PageCache::new(
            arch,
            Length::from_mb(2),
            PageType::PAGE_TABLE | PageType::READ_ONLY,
            TimedCacheValidator::new(coarsetime::Duration::from_secs(100)),
        );

        let mut mem_cache = CachedMemoryAccess::with(&mut dummy_mem, cache);

        let mut buf_1 = vec![0_u8; 64];
        mem_cache
            .phys_read_into(addr, buf_1.as_mut_slice())
            .unwrap();

        assert_eq!(buf_start, buf_1);
    }
    #[test]
    fn cache_phys_mem_diffpages() {
        let mut dummy_mem = DummyMemory::new(Length::from_mb(16));

        let mut buf_start = vec![0_u8; 64];
        for (i, item) in buf_start.iter_mut().enumerate() {
            *item = (i % 256) as u8;
        }

        let address = Address::from(0x5323);

        let addr1 =
            PhysicalAddress::with_page(address, PageType::from_writeable_bit(false), 0x1000.into());

        let addr2 =
            PhysicalAddress::with_page(address, PageType::from_writeable_bit(false), 0x100.into());

        dummy_mem
            .phys_write_raw(addr1, buf_start.as_slice())
            .unwrap();

        let cache = PageCache::with_page_size(
            Length::from(0x10),
            Length::from(0x10),
            PageType::PAGE_TABLE | PageType::READ_ONLY,
            TimedCacheValidator::new(coarsetime::Duration::from_secs(100)),
        );

        let mut mem_cache = CachedMemoryAccess::with(&mut dummy_mem, cache);

        let mut buf_1 = vec![0_u8; 64];
        mem_cache
            .phys_read_into(addr1, buf_1.as_mut_slice())
            .unwrap();

        assert_eq!(buf_start, buf_1);

        let mut buf_2 = vec![0_u8; 64];
        mem_cache
            .phys_read_into(addr2, buf_2.as_mut_slice())
            .unwrap();

        assert_eq!(buf_1, buf_2);

        let mut buf_3 = vec![0_u8; 64];
        mem_cache
            .phys_read_into(addr2, buf_3.as_mut_slice())
            .unwrap();

        assert_eq!(buf_2, buf_3);
    }

    #[test]
    fn writeback() {
        let mut dummy_mem = DummyMemory::new(Length::from_mb(16));
        let virt_size = Length::from_mb(8);
        let mut buf_start = vec![0_u8; 64];
        for (i, item) in buf_start.iter_mut().enumerate() {
            *item = (i % 256) as u8;
        }
        let (dtb, virt_base) = dummy_mem.alloc_dtb(virt_size, &buf_start);
        let arch = Architecture::X64;

        let cache = PageCache::new(
            arch,
            Length::from_mb(2),
            PageType::PAGE_TABLE | PageType::READ_ONLY,
            TimedCacheValidator::new(coarsetime::Duration::from_secs(100)),
        );

        let mut mem_cache = CachedMemoryAccess::with(&mut dummy_mem, cache);
        let mut virt_mem = VirtualFromPhysical::new(&mut mem_cache, arch, arch, dtb);

        let mut buf_1 = vec![0_u8; 64];
        virt_mem
            .virt_read_into(virt_base, buf_1.as_mut_slice())
            .unwrap();

        assert_eq!(buf_start, buf_1);
        buf_1[16..20].copy_from_slice(&[255, 255, 255, 255]);
        virt_mem
            .virt_write(virt_base + Length::from(16), &buf_1[16..20])
            .unwrap();

        let mut buf_2 = vec![0_u8; 64];
        virt_mem
            .virt_read_into(virt_base, buf_2.as_mut_slice())
            .unwrap();

        assert_eq!(buf_1, buf_2);
        assert_ne!(buf_2, buf_start);

        let mut buf_3 = vec![0_u8; 64];

        virt_mem
            .virt_read_into(virt_base, buf_3.as_mut_slice())
            .unwrap();
        assert_eq!(buf_2, buf_3);
    }
}
