use crate::architecture::Architecture;
use crate::error::{Error, Result};
use crate::mem::{PhysicalMemory, PhysicalReadIterator, PhysicalWriteIterator};
use crate::process::{OsProcessInfo, OsProcessModuleInfo};
use crate::types::{Address, Length};

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

pub struct DummyModule {
    base: Address,
    size: Length,
}

impl OsProcessModuleInfo for DummyModule {
    fn address(&self) -> Address {
        Address::INVALID
    }
    fn parent_process(&self) -> Address {
        Address::INVALID
    }

    fn base(&self) -> Address {
        self.base
    }
    fn size(&self) -> Length {
        self.size
    }
    fn name(&self) -> String {
        String::from("dummy.so")
    }
}

pub struct DummyProcess {
    address: Address,
    map_size: Length,
    pid: i32,
    dtb: Address,
}

impl DummyProcess {
    pub fn get_module(&self, min_size: Length) -> DummyModule {
        DummyModule {
            base: self.address
                + Length::from(thread_rng().gen_range(0, self.map_size.as_u64() / 2)),
            size: (thread_rng().gen_range(min_size.as_u64(), self.map_size.as_u64()) / 2).into(),
        }
    }
}

impl OsProcessInfo for DummyProcess {
    fn address(&self) -> Address {
        self.address
    }

    fn pid(&self) -> i32 {
        self.pid
    }

    fn name(&self) -> String {
        String::from("Dummy")
    }

    fn dtb(&self) -> Address {
        self.dtb
    }

    fn sys_arch(&self) -> Architecture {
        Architecture::X64
    }

    fn proc_arch(&self) -> Architecture {
        Architecture::X64
    }
}

pub struct DummyMemory {
    mem: Box<[u8]>,
    page_list: VecDeque<PageInfo>,
    pt_pages: Vec<PageInfo>,
    last_pid: i32,
}

impl PhysicalMemory for DummyMemory {
    fn phys_read_iter<'b, PI: PhysicalReadIterator<'b>>(&'b mut self, mut iter: PI) -> Result<()> {
        iter.try_for_each(move |(addr, out)| {
            if addr.address().as_usize() + out.len() <= self.mem.len() {
                out.copy_from_slice(&self.mem[addr.as_usize()..(addr.as_usize() + out.len())]);
                Ok(())
            } else {
                Err(Error::new("Read out of bounds"))
            }
        })
    }

    fn phys_write_iter<'b, PI: PhysicalWriteIterator<'b>>(
        &'b mut self,
        mut iter: PI,
    ) -> Result<()> {
        iter.try_for_each(move |(addr, data)| {
            if addr.address().as_usize() + data.len() <= self.mem.len() {
                self.mem[addr.as_usize()..(addr.as_usize() + data.len())].copy_from_slice(data);
                Ok(())
            } else {
                Err(Error::new("Write out of bounds"))
            }
        })
    }
}

unsafe impl<S> FrameAllocator<S> for DummyMemory
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

impl DummyMemory {
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
            last_pid: 0,
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

    pub fn alloc_process(&mut self, map_size: Length, test_buf: &[u8]) -> DummyProcess {
        let (dtb, address) = self.alloc_dtb(map_size, test_buf);

        self.last_pid += 1;

        DummyProcess {
            address,
            dtb,
            pid: self.last_pid,
            map_size,
        }
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
