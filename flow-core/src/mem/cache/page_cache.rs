use super::{CacheValidator, PageType};
use crate::architecture::Architecture;
use crate::error::Error;
use crate::iter::FlowIters;
use crate::mem::phys::{PhysicalReadData, PhysicalReadIterator};
use crate::mem::{AccessPhysicalMemory, AccessPhysicalMemoryExt};
use crate::page_chunks::PageChunksMut;
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
    pub fn new(
        arch: Architecture,
        size: Length,
        page_type_mask: PageType,
        mut validator: T,
    ) -> Self {
        let page_size = arch.page_size();
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

    fn cached_read_single<F: AccessPhysicalMemoryExt>(
        &mut self,
        mem: &mut F,
        addr: PhysicalAddress,
        out: &mut [u8],
    ) -> Result<(), Error> {
        // try read from cache or fall back
        if self.is_cached_page_type(addr.page_type()) {
            for (paddr, chunk) in PageChunksMut::create_from(out, addr.address(), self.page_size())
            {
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
        PageChunksMut::create_from(out, addr.address(), page_size).map(move |(paddr, chunk)| {
            (
                PhysicalAddress::with_page(paddr, addr.page_type(), addr.page_size()),
                chunk,
            )
        })
    }

    #[allow(clippy::never_loop)]
    pub fn cached_read<'a, F: AccessPhysicalMemory, PI: PhysicalReadIterator<'a>>(
        &'a mut self,
        mem: &'a mut F,
        iter: PI,
        arena: &Bump,
    ) -> Result<(), crate::Error> {
        let page_size = self.page_size;

        let mut iter = iter.double_peekable();

        if iter.is_next_last() {
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
                PageChunksMut::create_from(out, addr.address(), page_size).for_each(
                    |(paddr, chunk)| {
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
                    },
                );

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
