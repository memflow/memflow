use crate::architecture::Architecture;
use crate::mem::cache::page_cache::PageCache;
use crate::mem::cache::timed_validator::TimedCacheValidator;
use crate::mem::{AccessVirtualMemory, VirtualAddressTranslator};
use crate::mem::{PhysicalReadIterator, PhysicalWriteIterator};
use crate::types::{Address, Done, Length, PhysicalAddress, ToDo};
use crate::*;

use flow_derive::*;

use rand::seq::SliceRandom;
use rand::{thread_rng, Rng};
use std::collections::VecDeque;

use x86_64::{
    structures::paging,
    structures::paging::{
        mapper::{Mapper, OffsetPageTable},
        page::{PageSize, Size1GiB, Size2MiB, Size4KiB},
        page_table::{PageTable, PageTableFlags},
        FrameAllocator, PhysFrame,
    },
    PhysAddr, VirtAddr,
};

#[derive(Clone, Copy, Debug)]
enum X64PageSize {
    P4k = 0,
    P2m = 1,
    P1g = 2,
}

impl X64PageSize {
    fn to_len(self) -> Length {
        match self {
            X64PageSize::P4k => Length::from_kb(4),
            X64PageSize::P2m => Length::from_mb(2),
            X64PageSize::P1g => Length::from_gb(1),
        }
    }

    fn to_idx(self) -> usize {
        match self {
            X64PageSize::P4k => 0,
            X64PageSize::P2m => 1,
            X64PageSize::P1g => 2,
        }
    }

    fn from_idx(idx: usize) -> Self {
        match idx {
            2 => X64PageSize::P1g,
            1 => X64PageSize::P2m,
            _ => X64PageSize::P4k,
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct PageInfo {
    addr: Address,
    size: X64PageSize,
}

impl PageInfo {
    fn split_to_size(&self, new_size: X64PageSize) -> Vec<Self> {
        let mut ret = vec![];
        for o in 0..(self.size.to_len().as_usize() / new_size.to_len().as_usize()) {
            ret.push(PageInfo {
                addr: self.addr + new_size.to_len() * o,
                size: new_size,
            });
        }
        ret
    }

    fn split_down(&self) -> Vec<Self> {
        self.split_to_size(X64PageSize::from_idx(self.size.to_idx() - 1))
    }
}

#[derive(AccessVirtualMemory, VirtualAddressTranslator)]
pub struct TestMemory {
    mem: Box<[u8]>,
    page_list: VecDeque<PageInfo>,
    pt_pages: Vec<PageInfo>,
}

impl AccessPhysicalMemory for TestMemory {
    fn phys_read_raw_iter<'b, PI: PhysicalReadIterator<'b>>(
        &'b mut self,
        iter: PI,
    ) -> Box<dyn PhysicalReadIterator<'b>> {
        Box::new(iter.map(move |x| match x {
            ToDo((addr, out)) => Done(if addr.address.as_usize() + out.len() <= self.mem.len() {
                out.copy_from_slice(&self.mem[addr.as_usize()..(addr.as_usize() + out.len())]);
                Ok((addr, out))
            } else {
                Err(Error::new("Read out of bounds"))
            }),
            _ => x,
        }))
    }

    fn phys_write_raw_iter<'b, PI: PhysicalWriteIterator<'b>>(
        &'b mut self,
        iter: PI,
    ) -> Box<dyn PhysicalWriteIterator<'b>> {
        Box::new(iter.map(move |x| match x {
            ToDo((addr, data)) => Done(if addr.address.as_usize() + data.len() <= self.mem.len() {
                self.mem[addr.as_usize()..(addr.as_usize() + data.len())].copy_from_slice(data);
                Ok((addr, data))
            } else {
                Err(Error::new("Write out of bounds"))
            }),
            _ => x,
        }))
    }
}

unsafe impl<S> FrameAllocator<S> for TestMemory
where
    S: PageSize,
{
    fn allocate_frame(&mut self) -> Option<PhysFrame<S>> {
        let new_page = self.alloc_pt_page();
        match PhysFrame::from_start_address(PhysAddr::new(new_page.addr.as_u64())) {
            Ok(s) => Some(s),
            _ => None,
        }
    }
}

impl TestMemory {
    pub fn new(size: Length) -> Self {
        let mem = vec![0_u8; size.as_usize()].into_boxed_slice();

        let mut page_prelist = vec![];

        let mut i = Address::from(0);
        let size_addr = Address::from(size.as_u64());

        while i < size_addr {
            if let Some(page_info) = {
                if size_addr - i >= X64PageSize::P1g.to_len() {
                    Some(PageInfo {
                        addr: i,
                        size: X64PageSize::P1g,
                    })
                } else if size_addr - i >= X64PageSize::P2m.to_len() {
                    Some(PageInfo {
                        addr: i,
                        size: X64PageSize::P2m,
                    })
                } else if size_addr - i >= X64PageSize::P4k.to_len() {
                    Some(PageInfo {
                        addr: i,
                        size: X64PageSize::P4k,
                    })
                } else {
                    None
                }
            } {
                i += page_info.size.to_len();
                page_prelist.push(page_info);
            } else {
                break;
            }
        }

        let mut page_list: Vec<PageInfo> = vec![];

        let mut split = [2, 0, 0].to_vec();

        for _ in 0..2 {
            page_prelist.shuffle(&mut thread_rng());
            for i in page_prelist {
                let mut list = if split[i.size.to_idx()] == 0
                    || (split[i.size.to_idx()] != 2 && thread_rng().gen::<bool>())
                {
                    split[i.size.to_idx()] = std::cmp::max(split[i.size.to_idx()], 1);
                    i.split_down()
                } else {
                    [i].to_vec()
                };

                list.shuffle(&mut thread_rng());

                for o in list {
                    page_list.push(o);
                }
            }

            page_prelist = page_list.clone();
        }

        Self {
            mem,
            page_list: page_list.into(),
            pt_pages: vec![],
        }
    }

    //Given it's the tests, we will have a panic if out of mem
    fn alloc_pt_page(&mut self) -> PageInfo {
        if let Some(page) = self.pt_pages.pop() {
            page
        } else {
            self.pt_pages = self
                .page_list
                .pop_front()
                .unwrap()
                .split_to_size(X64PageSize::P4k);
            self.pt_pages.pop().unwrap()
        }
    }

    fn next_page_for_address(&mut self, _addr: Address) -> PageInfo {
        self.alloc_pt_page()
    }

    pub fn alloc_dtb(&mut self, map_size: Length, test_buf: &[u8]) -> (Address, Address) {
        let mut cur_len = Length::from(0);

        let dtb = self.alloc_pt_page();
        let virt_base =
            Address::from(thread_rng().gen_range(0x0001_0000_0000_u64, ((!0_u64) << 16) >> 16))
                .as_page_aligned(Length::from_gb(2));

        let mut pml4 = unsafe {
            &mut *(self
                .mem
                .as_mut_ptr()
                .add(dtb.addr.as_usize())
                .cast::<PageTable>())
        };
        *pml4 = PageTable::new();

        let mut pt_mapper =
            unsafe { OffsetPageTable::new(&mut pml4, VirtAddr::from_ptr(self.mem.as_ptr())) };

        while cur_len < map_size {
            let page_info = self.next_page_for_address(cur_len.as_u64().into());
            let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;

            if test_buf.len() >= (cur_len + page_info.size.to_len()).as_usize() {
                self.mem[page_info.addr.as_usize()
                    ..(page_info.addr + page_info.size.to_len()).as_usize()]
                    .copy_from_slice(
                        &test_buf
                            [cur_len.as_usize()..(cur_len + page_info.size.to_len()).as_usize()],
                    );
            } else if test_buf.len() > cur_len.as_usize() {
                self.mem[page_info.addr.as_usize()
                    ..(page_info.addr.as_usize() + test_buf.len() - cur_len.as_usize())]
                    .copy_from_slice(&test_buf[cur_len.as_usize()..]);
            }

            unsafe {
                match page_info.size {
                    X64PageSize::P1g => pt_mapper
                        .map_to(
                            paging::page::Page::<Size1GiB>::from_start_address_unchecked(
                                VirtAddr::new((virt_base + cur_len).as_u64()),
                            ),
                            PhysFrame::from_start_address_unchecked(PhysAddr::new(
                                page_info.addr.as_u64(),
                            )),
                            flags,
                            self,
                        )
                        .is_ok(),
                    X64PageSize::P2m => pt_mapper
                        .map_to(
                            paging::page::Page::<Size2MiB>::from_start_address_unchecked(
                                VirtAddr::new((virt_base + cur_len).as_u64()),
                            ),
                            PhysFrame::from_start_address_unchecked(PhysAddr::new(
                                page_info.addr.as_u64(),
                            )),
                            flags,
                            self,
                        )
                        .is_ok(),
                    X64PageSize::P4k => pt_mapper
                        .map_to(
                            paging::page::Page::<Size4KiB>::from_start_address_unchecked(
                                VirtAddr::new((virt_base + cur_len).as_u64()),
                            ),
                            PhysFrame::from_start_address_unchecked(PhysAddr::new(
                                page_info.addr.as_u64(),
                            )),
                            flags,
                            self,
                        )
                        .is_ok(),
                };
            }
            cur_len += page_info.size.to_len();
        }

        (dtb.addr, virt_base)
    }
}

#[test]
fn test_cached_mem() {
    let mut mem = TestMemory::new(Length::from_mb(512));
    let virt_size = Length::from_mb(8);
    let mut test_buf = vec![0_u64; virt_size.as_usize() / 8];

    for i in &mut test_buf {
        *i = thread_rng().gen::<u64>();
    }

    let test_buf =
        unsafe { std::slice::from_raw_parts(test_buf.as_ptr() as *const u8, virt_size.as_usize()) };

    let (dtb, virt_base) = mem.alloc_dtb(virt_size, &test_buf);
    let arch = Architecture::X64;

    let mut buf_nocache = vec![0_u8; test_buf.len()];
    mem.virt_read_raw_into(arch, dtb, virt_base, buf_nocache.as_mut_slice())
        .unwrap();

    assert_eq!(buf_nocache, test_buf);

    let cache = PageCache::new(
        arch,
        Length::from_mb(2),
        PageType::PAGE_TABLE | PageType::READ_ONLY,
        TimedCacheValidator::new(coarsetime::Duration::from_millis(100)),
    );
    let mut mem_cache = CachedMemoryAccess::with(&mut mem, cache);
    let mut buf_cache = vec![0_u8; buf_nocache.len()];
    mem_cache
        .virt_read_raw_into(arch, dtb, virt_base, buf_cache.as_mut_slice())
        .unwrap();

    assert_eq!(buf_nocache, buf_cache);
}

#[test]
fn test_cache_invalidity_cached() {
    let mut mem = TestMemory::new(Length::from_mb(512));
    let mem_ptr = &mut mem as *mut TestMemory;
    let virt_size = Length::from_mb(8);
    let mut buf_start = vec![0_u8; 64];
    for (i, item) in buf_start.iter_mut().enumerate() {
        *item = (i % 256) as u8;
    }
    let (dtb, virt_base) = mem.alloc_dtb(virt_size, &buf_start);
    let arch = Architecture::X64;

    let cache = PageCache::new(
        arch,
        Length::from_mb(2),
        PageType::PAGE_TABLE | PageType::READ_ONLY | PageType::WRITEABLE,
        TimedCacheValidator::new(coarsetime::Duration::from_secs(100)),
    );

    let mut mem_cache = CachedMemoryAccess::with(&mut mem, cache);

    //Modifying the memory from other channels should leave the cached page unchanged
    let mut cached_buf = vec![0_u8; 64];
    mem_cache
        .virt_read_raw_into(arch, dtb, virt_base, cached_buf.as_mut_slice())
        .unwrap();

    let mut write_buf = cached_buf.clone();
    write_buf[16..20].copy_from_slice(&[255, 255, 255, 255]);
    unsafe { mem_ptr.as_mut().unwrap() }
        .virt_write_raw(arch, dtb, virt_base, write_buf.as_slice())
        .unwrap();

    let mut check_buf = vec![0_u8; 64];
    mem_cache
        .virt_read_raw_into(arch, dtb, virt_base, check_buf.as_mut_slice())
        .unwrap();

    assert_eq!(cached_buf, check_buf);
    assert_ne!(check_buf, write_buf);
}

#[test]
fn test_cache_invalidity_non_cached() {
    let mut mem = TestMemory::new(Length::from_mb(512));
    let mem_ptr = &mut mem as *mut TestMemory;
    let virt_size = Length::from_mb(8);
    let mut buf_start = vec![0_u8; 64];
    for (i, item) in buf_start.iter_mut().enumerate() {
        *item = (i % 256) as u8;
    }
    let (dtb, virt_base) = mem.alloc_dtb(virt_size, &buf_start);
    let arch = Architecture::X64;

    //alloc_dtb creates a page table with all writeable pages, we disable cache for them
    let cache = PageCache::new(
        arch,
        Length::from_mb(2),
        PageType::PAGE_TABLE | PageType::READ_ONLY,
        TimedCacheValidator::new(coarsetime::Duration::from_secs(100)),
    );

    let mut mem_cache = CachedMemoryAccess::with(&mut mem, cache);

    //Modifying the memory from other channels should leave the cached page unchanged
    let mut cached_buf = vec![0_u8; 64];
    mem_cache
        .virt_read_raw_into(arch, dtb, virt_base, cached_buf.as_mut_slice())
        .unwrap();

    let mut write_buf = cached_buf.clone();
    write_buf[16..20].copy_from_slice(&[255, 255, 255, 255]);
    unsafe { mem_ptr.as_mut().unwrap() }
        .virt_write_raw(arch, dtb, virt_base, write_buf.as_slice())
        .unwrap();

    let mut check_buf = vec![0_u8; 64];
    mem_cache
        .virt_read_raw_into(arch, dtb, virt_base, check_buf.as_mut_slice())
        .unwrap();

    assert_ne!(cached_buf, check_buf);
    assert_eq!(check_buf, write_buf);
}

#[test]
fn test_writeback() {
    let mut mem = TestMemory::new(Length::from_mb(16));
    let virt_size = Length::from_mb(8);
    let mut buf_start = vec![0_u8; 64];
    for (i, item) in buf_start.iter_mut().enumerate() {
        *item = (i % 256) as u8;
    }
    let (dtb, virt_base) = mem.alloc_dtb(virt_size, &buf_start);
    let arch = Architecture::X64;

    let cache = PageCache::new(
        arch,
        Length::from_mb(2),
        PageType::PAGE_TABLE | PageType::READ_ONLY,
        TimedCacheValidator::new(coarsetime::Duration::from_secs(100)),
    );

    let mut mem = CachedMemoryAccess::with(&mut mem, cache);

    let mut buf_1 = vec![0_u8; 64];
    mem.virt_read_into(arch, dtb, virt_base, buf_1.as_mut_slice())
        .unwrap();

    assert_eq!(buf_start, buf_1);
    buf_1[16..20].copy_from_slice(&[255, 255, 255, 255]);

    mem.virt_write(arch, dtb, virt_base + Length::from(16), &buf_1[16..20])
        .unwrap();

    let mut buf_2 = vec![0_u8; 64];
    mem.virt_read_into(arch, dtb, virt_base, buf_2.as_mut_slice())
        .unwrap();

    assert_eq!(buf_1, buf_2);
    assert_ne!(buf_2, buf_start);

    let mut buf_3 = vec![0_u8; 64];

    mem.virt_read_into(arch, dtb, virt_base, buf_3.as_mut_slice())
        .unwrap();
    assert_eq!(buf_2, buf_3);
}

#[test]
fn test_vtop() {
    let mut mem = TestMemory::new(Length::from_mb(512));
    let virt_size = Length::from_mb(8);
    let (dtb, virt_base) = mem.alloc_dtb(virt_size, &[]);
    let arch = Architecture::X64;

    assert_eq!(mem.virt_to_phys(arch, dtb, virt_base).is_ok(), true);
    assert_eq!(
        arch.virt_to_phys(
            &mut mem,
            dtb,
            virt_base + Length::from(virt_size.as_usize() / 2),
        )
        .is_ok(),
        true
    );
    assert_eq!(
        mem.virt_to_phys(arch, dtb, virt_base - Length::from_mb(1))
            .is_ok(),
        false
    );
    assert_eq!(
        mem.virt_to_phys(arch, dtb, virt_base + virt_size).is_ok(),
        false
    );
}

#[test]
fn test_virt_read_small() {
    let mut mem = TestMemory::new(Length::from_mb(2));
    let mut buf = vec![0u8; 256];
    for (i, item) in buf.iter_mut().enumerate() {
        *item = i as u8;
    }
    let (dtb, virt_base) = mem.alloc_dtb(buf.len().into(), &buf);

    let mut out = vec![0u8; buf.len()];
    mem.virt_read_into(Architecture::X64, dtb, virt_base, &mut out[..])
        .unwrap();
    assert_eq!(buf.len(), out.len());
    assert_eq!(buf, out);
}

#[test]
fn test_virt_write_small() {
    let mut mem = TestMemory::new(Length::from_mb(2));
    let mut buf = vec![0u8; 256];
    let mut input = vec![0u8; buf.len()];
    for (i, item) in input.iter_mut().enumerate() {
        *item = i as u8;
    }
    let (dtb, virt_base) = mem.alloc_dtb(input.len().into(), &input);
    mem.virt_write(Architecture::X64, dtb, virt_base, &input[..])
        .unwrap();
    mem.virt_read_into(Architecture::X64, dtb, virt_base, &mut buf[..])
        .unwrap();
    assert_eq!(buf.len(), input.len());
    assert_eq!(buf, input);
}

#[test]
fn test_virt_read_small_shifted() {
    let mut mem = TestMemory::new(Length::from_mb(2));
    let mut buf = vec![0u8; 256];
    for (i, item) in buf.iter_mut().enumerate() {
        *item = i as u8;
    }
    let (dtb, virt_base) = mem.alloc_dtb(buf.len().into(), &buf);

    let mut out = vec![0u8; buf.len() - 128];
    mem.virt_read_into(
        Architecture::X64,
        dtb,
        virt_base + Length::from(128),
        &mut out[..],
    )
    .unwrap();
    assert_eq!(buf[128..].to_vec().len(), out.len());
    assert_eq!(buf[128..].to_vec(), out);
}

#[test]
fn test_virt_write_small_shifted() {
    let mut mem = TestMemory::new(Length::from_mb(2));
    let mut buf = vec![0u8; 128];
    let mut input = vec![0u8; buf.len()];
    for (i, item) in input.iter_mut().enumerate() {
        *item = i as u8;
    }
    let (dtb, virt_base) = mem.alloc_dtb(input.len().into(), &input);
    mem.virt_write(
        Architecture::X64,
        dtb,
        virt_base + Length::from(128),
        &input[..],
    )
    .unwrap();
    mem.virt_read_into(
        Architecture::X64,
        dtb,
        virt_base + Length::from(128),
        &mut buf[..],
    )
    .unwrap();
    assert_eq!(buf.to_vec().len(), input.len());
    assert_eq!(buf.to_vec(), input);
}

#[test]
fn test_virt_read_medium() {
    let mut mem = TestMemory::new(Length::from_mb(2));
    let mut buf = vec![0u8; 0x1000];
    for (i, item) in buf.iter_mut().enumerate() {
        *item = i as u8;
    }
    let (dtb, virt_base) = mem.alloc_dtb(buf.len().into(), &buf);

    let mut out = vec![0u8; buf.len()];
    mem.virt_read_into(Architecture::X64, dtb, virt_base, &mut out[..])
        .unwrap();
    assert_eq!(buf.len(), out.len());
    assert_eq!(buf, out);
}

#[test]
fn test_virt_write_medium() {
    let mut mem = TestMemory::new(Length::from_mb(2));
    let mut buf = vec![0u8; 0x1000];
    let mut input = vec![0u8; buf.len()];
    for (i, item) in input.iter_mut().enumerate() {
        *item = i as u8;
    }
    let (dtb, virt_base) = mem.alloc_dtb(input.len().into(), &input);
    mem.virt_write(Architecture::X64, dtb, virt_base, &input[..])
        .unwrap();
    mem.virt_read_into(Architecture::X64, dtb, virt_base, &mut buf[..])
        .unwrap();
    assert_eq!(buf.len(), input.len());
    assert_eq!(buf, input);
}

#[test]
fn test_virt_read_medium_shifted() {
    let mut mem = TestMemory::new(Length::from_mb(2));
    let mut buf = vec![0u8; 0x1000];
    for (i, item) in buf.iter_mut().enumerate() {
        *item = i as u8;
    }
    let (dtb, virt_base) = mem.alloc_dtb(buf.len().into(), &buf);

    let mut out = vec![0u8; buf.len() - 0x100];
    mem.virt_read_into(
        Architecture::X64,
        dtb,
        virt_base + Length::from(0x100),
        &mut out[..],
    )
    .unwrap();
    assert_eq!(buf[0x100..].to_vec().len(), out.len());
    assert_eq!(buf[0x100..].to_vec(), out);
}

#[test]
fn test_virt_write_medium_shifted() {
    let mut mem = TestMemory::new(Length::from_mb(2));
    let mut buf = vec![0u8; 0x1000 - 0x100];
    let mut input = vec![0u8; buf.len()];
    for (i, item) in input.iter_mut().enumerate() {
        *item = i as u8;
    }
    let (dtb, virt_base) = mem.alloc_dtb(input.len().into(), &input);
    mem.virt_write(
        Architecture::X64,
        dtb,
        virt_base + Length::from(0x100),
        &input[..],
    )
    .unwrap();
    mem.virt_read_into(
        Architecture::X64,
        dtb,
        virt_base + Length::from(0x100),
        &mut buf[..],
    )
    .unwrap();
    assert_eq!(buf.to_vec().len(), input.len());
    assert_eq!(buf.to_vec(), input);
}

#[test]
fn test_virt_read_big() {
    let mut mem = TestMemory::new(Length::from_mb(2));
    let mut buf = vec![0u8; 0x1000 * 16];
    for (i, item) in buf.iter_mut().enumerate() {
        *item = i as u8;
    }
    let (dtb, virt_base) = mem.alloc_dtb(buf.len().into(), &buf);

    let mut out = vec![0u8; buf.len()];
    mem.virt_read_into(Architecture::X64, dtb, virt_base, &mut out[..])
        .unwrap();
    assert_eq!(buf.len(), out.len());
    assert_eq!(buf, out);
}

#[test]
fn test_virt_write_big() {
    let mut mem = TestMemory::new(Length::from_mb(2));
    let mut buf = vec![0u8; 0x1000 * 16];
    let mut input = vec![0u8; buf.len()];
    for (i, item) in input.iter_mut().enumerate() {
        *item = i as u8;
    }
    let (dtb, virt_base) = mem.alloc_dtb(input.len().into(), &input);
    mem.virt_write(Architecture::X64, dtb, virt_base, &input[..])
        .unwrap();
    mem.virt_read_into(Architecture::X64, dtb, virt_base, &mut buf[..])
        .unwrap();
    assert_eq!(buf.len(), input.len());
    assert_eq!(buf, input);
}

#[test]
fn test_virt_read_big_shifted() {
    let mut mem = TestMemory::new(Length::from_mb(2));
    let mut buf = vec![0u8; 0x1000 * 16];
    for (i, item) in buf.iter_mut().enumerate() {
        *item = i as u8;
    }
    let (dtb, virt_base) = mem.alloc_dtb(buf.len().into(), &buf);

    let mut out = vec![0u8; buf.len() - 0x100];
    mem.virt_read_into(
        Architecture::X64,
        dtb,
        virt_base + Length::from(0x100),
        &mut out[..],
    )
    .unwrap();
    assert_eq!(buf[0x100..].to_vec().len(), out.len());
    assert_eq!(buf[0x100..].to_vec(), out);
}

#[test]
fn test_virt_write_big_shifted() {
    let mut mem = TestMemory::new(Length::from_mb(2));
    let mut buf = vec![0u8; 0x1000 * 16 - 0x100];
    let mut input = vec![0u8; buf.len()];
    for (i, item) in input.iter_mut().enumerate() {
        *item = i as u8;
    }
    let (dtb, virt_base) = mem.alloc_dtb(input.len().into(), &input);
    mem.virt_write(
        Architecture::X64,
        dtb,
        virt_base + Length::from(0x100),
        &input[..],
    )
    .unwrap();
    mem.virt_read_into(
        Architecture::X64,
        dtb,
        virt_base + Length::from(0x100),
        &mut buf[..],
    )
    .unwrap();
    assert_eq!(buf.to_vec().len(), input.len());
    assert_eq!(buf.to_vec(), input);
}
