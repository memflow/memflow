use crate::architecture::x86::x64;
use crate::architecture::Architecture;
use crate::error::{Error, Result};
use crate::mem::virt_mem::virt_from_phys::VirtualFromPhysical;
use crate::mem::{PhysicalMemory, PhysicalReadData, PhysicalWriteData, VirtualMemory};
use crate::process::{OsProcessInfo, OsProcessModuleInfo};
use crate::types::{size, Address};

use rand::seq::SliceRandom;
use rand::{thread_rng, Rng, SeedableRng};
use rand_xorshift::XorShiftRng;
use std::collections::VecDeque;

use x86_64::{
    structures::paging,
    structures::paging::{
        mapper::{Mapper, MapperAllSizes, OffsetPageTable},
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
    fn to_size(self) -> usize {
        match self {
            X64PageSize::P4k => size::kb(4),
            X64PageSize::P2m => size::mb(2),
            X64PageSize::P1g => size::gb(1),
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
        for o in 0..(self.size.to_size() / new_size.to_size()) {
            ret.push(PageInfo {
                addr: self.addr + new_size.to_size() * o,
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
    size: usize,
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

    fn size(&self) -> usize {
        self.size
    }

    fn name(&self) -> String {
        String::from("dummy.so")
    }
}

pub struct DummyProcess {
    address: Address,
    map_size: usize,
    pid: i32,
    dtb: Address,
}

impl DummyProcess {
    pub fn get_module(&self, min_size: usize) -> DummyModule {
        DummyModule {
            base: self.address + thread_rng().gen_range(0, self.map_size / 2),
            size: (thread_rng().gen_range(min_size, self.map_size) / 2),
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

    fn sys_arch(&self) -> &dyn Architecture {
        x64::ARCH
    }

    fn proc_arch(&self) -> &dyn Architecture {
        x64::ARCH
    }
}

#[derive(Clone)]
pub struct DummyMemory {
    mem: Box<[u8]>,
    page_list: VecDeque<PageInfo>,
    pt_pages: Vec<PageInfo>,
    last_pid: i32,
    rng: XorShiftRng,
}

impl PhysicalMemory for DummyMemory {
    fn phys_read_raw_list(&mut self, data: &mut [PhysicalReadData]) -> Result<()> {
        data.iter_mut().try_for_each(move |(addr, out)| {
            if addr.address().as_usize() + out.len() <= self.mem.len() {
                out.copy_from_slice(&self.mem[addr.as_usize()..(addr.as_usize() + out.len())]);
                Ok(())
            } else {
                Err(Error::PhysicalMemory("read out of bounds"))
            }
        })
    }

    fn phys_write_raw_list(&mut self, data: &[PhysicalWriteData]) -> Result<()> {
        data.iter().try_for_each(move |(addr, data)| {
            if addr.address().as_usize() + data.len() <= self.mem.len() {
                self.mem[addr.as_usize()..(addr.as_usize() + data.len())].copy_from_slice(data);
                Ok(())
            } else {
                Err(Error::PhysicalMemory("write out of bounds"))
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
    pub fn new_and_dtb(size: usize, virt_size: usize, buffer: &[u8]) -> (Self, Address, Address) {
        let mut ret = Self::new(size);
        let (dtb, virt_base) = ret.alloc_dtb(virt_size, buffer);
        (ret, dtb, virt_base)
    }

    pub fn new_virt(size: usize, virt_size: usize, buffer: &[u8]) -> (impl VirtualMemory, Address) {
        let (ret, dtb, virt_base) = Self::new_and_dtb(size, virt_size, buffer);
        let virt = VirtualFromPhysical::new(ret, x64::ARCH, x64::new_translator(dtb));
        (virt, virt_base)
    }

    pub fn new(size: usize) -> Self {
        Self::with_rng(size, SeedableRng::from_rng(thread_rng()).unwrap())
    }

    pub fn with_seed(size: usize, seed: u64) -> Self {
        Self::with_rng(size, SeedableRng::seed_from_u64(seed))
    }

    pub fn with_rng(size: usize, mut rng: XorShiftRng) -> Self {
        let mem = vec![0_u8; size].into_boxed_slice();

        let mut page_prelist = vec![];

        let mut i = Address::from(0);
        let size_addr = Address::from(size);

        while i < size_addr {
            if let Some(page_info) = {
                if size_addr - i >= X64PageSize::P1g.to_size() {
                    Some(PageInfo {
                        addr: i,
                        size: X64PageSize::P1g,
                    })
                } else if size_addr - i >= X64PageSize::P2m.to_size() {
                    Some(PageInfo {
                        addr: i,
                        size: X64PageSize::P2m,
                    })
                } else if size_addr - i >= X64PageSize::P4k.to_size() {
                    Some(PageInfo {
                        addr: i,
                        size: X64PageSize::P4k,
                    })
                } else {
                    None
                }
            } {
                i += page_info.size.to_size();
                page_prelist.push(page_info);
            } else {
                break;
            }
        }

        let mut page_list: Vec<PageInfo> = vec![];

        let mut split = [2, 0, 0].to_vec();

        for _ in 0..2 {
            page_prelist.shuffle(&mut rng);
            for i in page_prelist {
                let mut list = if split[i.size.to_idx()] == 0
                    || (split[i.size.to_idx()] != 2 && rng.gen::<bool>())
                {
                    split[i.size.to_idx()] = std::cmp::max(split[i.size.to_idx()], 1);
                    i.split_down()
                } else {
                    [i].to_vec()
                };

                list.shuffle(&mut rng);

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
            rng,
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

    pub fn alloc_process(&mut self, map_size: usize, test_buf: &[u8]) -> DummyProcess {
        let (dtb, address) = self.alloc_dtb(map_size, test_buf);

        self.last_pid += 1;

        DummyProcess {
            address,
            dtb,
            pid: self.last_pid,
            map_size,
        }
    }

    pub fn vtop(&mut self, dtb_base: Address, virt_addr: Address) -> Option<Address> {
        let mut pml4 = unsafe {
            &mut *(self
                .mem
                .as_mut_ptr()
                .add(dtb_base.as_usize())
                .cast::<PageTable>())
        };

        let pt_mapper =
            unsafe { OffsetPageTable::new(&mut pml4, VirtAddr::from_ptr(self.mem.as_ptr())) };

        match pt_mapper.translate_addr(VirtAddr::new(virt_addr.as_u64())) {
            None => None,
            Some(addr) => Some(Address::from(addr.as_u64())),
        }
    }

    pub fn alloc_dtb(&mut self, map_size: usize, test_buf: &[u8]) -> (Address, Address) {
        let virt_base = (Address::null()
            + self
                .rng
                .gen_range(0x0001_0000_0000_usize, ((!0_usize) << 20) >> 20))
        .as_page_aligned(size::gb(2));

        (
            self.alloc_dtb_const_base(virt_base, map_size, test_buf),
            virt_base,
        )
    }

    pub fn alloc_dtb_const_base(
        &mut self,
        virt_base: Address,
        map_size: usize,
        test_buf: &[u8],
    ) -> Address {
        let mut cur_len = 0;

        let dtb = self.alloc_pt_page();

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
            let page_info = self.next_page_for_address(cur_len.into());
            let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;

            if test_buf.len() >= (cur_len + page_info.size.to_size()) {
                self.mem[page_info.addr.as_usize()
                    ..(page_info.addr + page_info.size.to_size()).as_usize()]
                    .copy_from_slice(&test_buf[cur_len..(cur_len + page_info.size.to_size())]);
            } else if test_buf.len() > cur_len {
                self.mem[page_info.addr.as_usize()
                    ..(page_info.addr.as_usize() + test_buf.len() - cur_len)]
                    .copy_from_slice(&test_buf[cur_len..]);
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
            cur_len += page_info.size.to_size();
        }

        dtb.addr
    }
}
