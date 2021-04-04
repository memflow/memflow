use super::{DummyMemory, DummyProcessInfo};
use crate::architecture::ArchitectureIdent;
use crate::error::{Error, ErrorKind, ErrorOrigin, Result};
use crate::mem::virt_mem::VirtualDMA;
use crate::mem::PhysicalMemory;
use crate::os::{ModuleInfo, OSInfo, ProcessInfo, PID};
use crate::plugins::{
    create_bare,
    os::{MUOSInstance, OSDescriptor},
    Args, COption, ConnectorInstance, OSInstance, MEMFLOW_PLUGIN_VERSION,
};
use crate::types::ReprCStr;
use crate::types::{size, Address};
use log::Level;
use rand::seq::SliceRandom;
use rand::{thread_rng, Rng, SeedableRng};
use rand_xorshift::XorShiftRng;
use std::collections::VecDeque;

use crate::architecture::x86::x64;
use crate::architecture::x86::X86ScopedVirtualTranslate;

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

pub struct DummyOS {
    mem: DummyMemory,
    page_list: VecDeque<PageInfo>,
    pt_pages: Vec<PageInfo>,
    last_pid: PID,
    rng: XorShiftRng,
    processes: Vec<DummyProcessInfo>,
    info: OSInfo,
}

impl Clone for DummyOS {
    fn clone(&self) -> Self {
        Self {
            mem: self.mem.clone(),
            page_list: VecDeque::new(),
            pt_pages: vec![],
            last_pid: self.last_pid,
            rng: self.rng.clone(),
            processes: self.processes.clone(),
            info: self.info.clone(),
        }
    }
}

impl AsMut<DummyMemory> for DummyOS {
    fn as_mut(&mut self) -> &mut DummyMemory {
        &mut self.mem
    }
}

unsafe impl<S> FrameAllocator<S> for DummyOS
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

impl DummyOS {
    pub fn new_and_dtb(
        mem: DummyMemory,
        virt_size: usize,
        buffer: &[u8],
    ) -> (Self, Address, Address) {
        let mut ret = Self::new(mem);
        let (dtb, virt_base) = ret.alloc_dtb(virt_size, buffer);
        (ret, dtb, virt_base)
    }

    pub fn into_inner(self) -> DummyMemory {
        self.mem
    }

    pub fn quick_process(virt_size: usize, buffer: &[u8]) -> <Self as OSInner>::IntoProcessType {
        let mem = DummyMemory::new(virt_size + size::mb(2));
        let mut os = Self::new(mem);
        let pid = os.alloc_process(virt_size, buffer);
        os.into_process_by_pid(pid).unwrap()
    }

    /*pub fn new_virt(size: usize, virt_size: usize, buffer: &[u8]) -> (impl VirtualMemory, Address) {
        let (ret, dtb, virt_base) = Self::new_and_dtb(size, virt_size, buffer);
        let virt = VirtualDMA::new(ret.mem, x64::ARCH, x64::new_translator(dtb));
        (virt, virt_base)
    }*/

    pub fn new(mem: DummyMemory) -> Self {
        Self::with_rng(mem, SeedableRng::from_rng(thread_rng()).unwrap())
    }

    pub fn with_seed(mem: DummyMemory, seed: u64) -> Self {
        Self::with_rng(mem, SeedableRng::seed_from_u64(seed))
    }

    pub fn with_rng(mem: DummyMemory, mut rng: XorShiftRng) -> Self {
        let mut page_prelist = vec![];

        let mut i = Address::from(0);
        let size_addr = Address::from(mem.metadata().size);

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
            processes: vec![],
            info: OSInfo {
                base: Address::INVALID,
                size: 0,
                arch: ArchitectureIdent::X86(64, false),
            },
        }
    }

    pub fn vtop(&mut self, dtb_base: Address, virt_addr: Address) -> Option<Address> {
        let mut pml4 = unsafe {
            &mut *(self
                .mem
                .buf
                .as_ptr()
                .add(dtb_base.as_usize())
                .cast::<PageTable>() as *mut _)
        };

        let pt_mapper =
            unsafe { OffsetPageTable::new(&mut pml4, VirtAddr::from_ptr(self.mem.buf.as_ptr())) };

        match pt_mapper.translate_addr(VirtAddr::new(virt_addr.as_u64())) {
            None => None,
            Some(addr) => Some(Address::from(addr.as_u64())),
        }
    }

    pub fn alloc_process(&mut self, map_size: usize, test_buf: &[u8]) -> PID {
        let (dtb, address) = self.alloc_dtb(map_size, test_buf);

        self.last_pid += 1;

        let proc = DummyProcessInfo {
            info: ProcessInfo {
                address,
                pid: self.last_pid,
                name: "Dummy".into(),
                sys_arch: x64::ARCH.ident(),
                proc_arch: x64::ARCH.ident(),
            },
            dtb,
            map_size,
            modules: vec![],
        };

        let ret = proc.info.pid;

        self.processes.push(proc);

        ret
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
                .buf
                .as_ptr()
                .add(dtb.addr.as_usize())
                .cast::<PageTable>() as *mut _)
        };
        *pml4 = PageTable::new();

        let mut pt_mapper =
            unsafe { OffsetPageTable::new(&mut pml4, VirtAddr::from_ptr(self.mem.buf.as_ptr())) };

        while cur_len < map_size {
            let page_info = self.next_page_for_address(cur_len.into());
            let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;

            if test_buf.len() >= (cur_len + page_info.size.to_size()) {
                self.mem
                    .phys_write_raw(
                        page_info.addr.into(),
                        &test_buf[cur_len..(cur_len + page_info.size.to_size())],
                    )
                    .unwrap();
            } else if test_buf.len() > cur_len {
                self.mem
                    .phys_write_raw(page_info.addr.into(), &test_buf[cur_len..])
                    .unwrap();
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
}

use super::process::DummyProcess;
use crate::mem::DirectTranslate;
use crate::os::{AddressCallback, OSInner};

pub type DummyVirtMem<T> = VirtualDMA<T, DirectTranslate, X86ScopedVirtualTranslate>;

impl<'a> OSInner<'a> for DummyOS {
    type ProcessType = DummyProcess<DummyVirtMem<&'a mut DummyMemory>>;
    type IntoProcessType = DummyProcess<DummyVirtMem<DummyMemory>>;

    /// Walks a process list and calls a callback for each process structure address
    ///
    /// The callback is fully opaque. We need this style so that C FFI can work seamlessly.
    fn process_address_list_callback(&mut self, mut callback: AddressCallback) -> Result<()> {
        self.processes
            .iter()
            .take_while(|p| callback.call(p.info.address))
            .for_each(|_| {});

        Ok(())
    }

    /// Find process information by its internal address
    fn process_info_by_address(&mut self, address: Address) -> Result<ProcessInfo> {
        self.processes
            .iter()
            .find(|p| p.info.address == address)
            .ok_or(Error(ErrorOrigin::OSLayer, ErrorKind::ProcessNotFound))
            .map(|p| p.info.clone())
    }

    /// Creates a process by its internal address
    ///
    /// It will share the underlying memory resources
    fn process_by_info(&'a mut self, info: ProcessInfo) -> Result<Self::ProcessType> {
        let proc = self
            .processes
            .iter()
            .find(|p| p.info.address == info.address)
            .ok_or(Error(ErrorOrigin::OSLayer, ErrorKind::InvalidProcessInfo))?
            .clone();
        Ok(DummyProcess {
            mem: VirtualDMA::new(&mut self.mem, x64::ARCH, x64::new_translator(proc.dtb)),
            proc,
        })
    }

    /// Creates a process by its internal address
    ///
    /// It will consume the kernel and not affect memory usage
    ///
    /// If no process with the specified address can be found this function will return an Error.
    ///
    /// This function can be useful for quickly accessing a process.
    fn into_process_by_info(self, info: ProcessInfo) -> Result<Self::IntoProcessType> {
        let proc = self
            .processes
            .iter()
            .find(|p| p.info.address == info.address)
            .ok_or(Error(ErrorOrigin::OSLayer, ErrorKind::InvalidProcessInfo))?
            .clone();
        Ok(DummyProcess {
            mem: VirtualDMA::new(self.mem, x64::ARCH, x64::new_translator(proc.dtb)),
            proc,
        })
    }

    /// Walks the kernel module list and calls the provided callback for each module structure
    /// address
    ///
    /// # Arguments
    /// * `callback` - where to pass each matching module to. This is an opaque callback.
    fn module_address_list_callback(&mut self, _callback: AddressCallback) -> Result<()> {
        Ok(())
    }

    /// Retrieves a module by its structure address
    ///
    /// # Arguments
    /// * `address` - address where module's information resides in
    fn module_by_address(&mut self, _address: Address) -> Result<ModuleInfo> {
        Err(Error(ErrorOrigin::OSLayer, ErrorKind::ModuleNotFound))
    }

    /// Retrieves the kernel info
    fn info(&self) -> &OSInfo {
        &self.info
    }
}

#[doc(hidden)]
#[no_mangle]
pub static MEMFLOW_OS_DUMMY: OSDescriptor = OSDescriptor {
    plugin_version: MEMFLOW_PLUGIN_VERSION,
    name: "dummy",
    version: env!("CARGO_PKG_VERSION"),
    description: "Dummy testing OS",
    create: mf_create,
};

#[doc(hidden)]
extern "C" fn mf_create(
    args: &ReprCStr,
    mem: COption<ConnectorInstance>,
    log_level: i32,
    out: &mut MUOSInstance,
) -> i32 {
    create_bare(args, mem.into(), log_level, out, build_dummy)
}

pub fn build_dummy(
    args: &Args,
    _mem: Option<ConnectorInstance>,
    _log_level: Level,
) -> Result<OSInstance> {
    let size = super::mem::parse_size(args)?;
    let mem = DummyMemory::new(size);
    let os = DummyOS::new(mem);

    let instance = OSInstance::builder(os).build();
    Ok(instance)
}
