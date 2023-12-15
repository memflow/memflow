use super::mem::*;
use super::process::*;

use crate::architecture::ArchitectureIdent;
use crate::error::{Error, ErrorKind, ErrorOrigin, Result};
use crate::mem::{phys_mem::*, virt_mem::*, *};
use crate::os::{process::*, root::*, *};
use crate::plugins::{self, *};
use crate::types::{clamp_to_usize, imem, mem, size, umem, Address};

use crate::cglue::*;
use rand::seq::SliceRandom;
use rand::{Rng, SeedableRng};
use rand_xorshift::XorShiftRng;
use std::collections::VecDeque;
use std::convert::TryInto;

use crate::architecture::x86::{x64, X86VirtualTranslate};

use x86_64::{
    structures::paging,
    structures::paging::{
        mapper::Mapper,
        page::{PageSize, Size1GiB, Size2MiB, Size4KiB},
        page_table::{PageTable, PageTableFlags},
        FrameAllocator, PhysFrame, Translate,
    },
    PhysAddr, VirtAddr,
};

use super::OffsetPageTable;

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
                addr: self.addr + new_size.to_size() as umem * o as umem,
                size: new_size,
            });
        }
        ret
    }

    fn split_down(&self) -> Vec<Self> {
        self.split_to_size(X64PageSize::from_idx(self.size.to_idx() - 1))
    }
}

cglue_impl_group!(DummyOs, OsInstance, PhysicalMemory);

pub struct DummyOs {
    mem: DummyMemory,
    page_list: VecDeque<PageInfo>,
    pt_pages: Vec<PageInfo>,
    last_pid: Pid,
    rng: XorShiftRng,
    processes: Vec<DummyProcessInfo>,
    info: OsInfo,
}

impl Clone for DummyOs {
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

impl AsMut<DummyMemory> for DummyOs {
    fn as_mut(&mut self) -> &mut DummyMemory {
        &mut self.mem
    }
}

unsafe impl<S> FrameAllocator<S> for DummyOs
where
    S: PageSize,
{
    fn allocate_frame(&mut self) -> Option<PhysFrame<S>> {
        let new_page = self.alloc_pt_page();
        match PhysFrame::from_start_address(PhysAddr::new(new_page.addr.to_umem() as u64)) {
            Ok(s) => Some(s),
            _ => None,
        }
    }
}

impl DummyOs {
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

    pub fn quick_process(virt_size: usize, buffer: &[u8]) -> <Self as Os>::IntoProcessType {
        let mem = DummyMemory::new(virt_size + size::mb(2));
        let mut os = Self::new(mem);
        let pid = os.alloc_process(virt_size, buffer);
        os.into_process_by_pid(pid).unwrap()
    }

    /// Creates a new DummyOs object with a fixed default seed
    ///
    /// Note:
    ///
    /// Using a fixed seed for the rng will provide reproducability throughout test cases.
    pub fn new(mem: DummyMemory) -> Self {
        Self::with_seed(mem, 1)
    }

    /// Creates a new DummyOs object with the given seed as a starting value for the RNG
    pub fn with_seed(mem: DummyMemory, seed: u64) -> Self {
        Self::with_rng(mem, SeedableRng::seed_from_u64(seed))
    }

    /// Creates a new DummyOs object with the given RNG.
    ///
    /// Note:
    ///
    /// The RNG has to be of type `XorShiftRng`.
    pub fn with_rng(mem: DummyMemory, mut rng: XorShiftRng) -> Self {
        let mut page_prelist = vec![];

        let mut i = Address::null();
        let size_addr = mem.metadata().max_address + 1_usize;

        while i < size_addr {
            if let Some(page_info) = {
                if size_addr - i >= X64PageSize::P1g.to_size() as imem {
                    Some(PageInfo {
                        addr: i,
                        size: X64PageSize::P1g,
                    })
                } else if size_addr - i >= X64PageSize::P2m.to_size() as imem {
                    Some(PageInfo {
                        addr: i,
                        size: X64PageSize::P2m,
                    })
                } else if size_addr - i >= X64PageSize::P4k.to_size() as imem {
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
            info: OsInfo {
                base: Address::INVALID,
                size: 0,
                arch: ArchitectureIdent::X86(64, false),
            },
        }
    }

    pub fn vtop(&mut self, dtb_base: Address, virt_addr: Address) -> Option<Address> {
        let pml4 = unsafe {
            &mut *(self
                .mem
                .buf_ptr()
                .add(dtb_base.to_umem().try_into().unwrap())
                .cast::<PageTable>() as *mut _)
        };

        let pt_mapper =
            unsafe { OffsetPageTable::new(pml4, VirtAddr::from_ptr(self.mem.buf_ptr())) };

        pt_mapper
            .translate_addr(VirtAddr::new(virt_addr.to_umem() as u64))
            .map(|addr| addr.as_u64().into())
    }

    fn internal_alloc_process(&mut self, map_size: usize, test_buf: &[u8]) -> DummyProcessInfo {
        let (dtb, address) = self.alloc_dtb(map_size, test_buf);

        self.last_pid += 1;

        DummyProcessInfo {
            info: ProcessInfo {
                address,
                pid: self.last_pid,
                state: ProcessState::Alive,
                name: "Dummy".into(),
                path: "/some/dummy".into(),
                command_line: "/some/dummy --dummyarg".into(),
                sys_arch: x64::ARCH.ident(),
                proc_arch: x64::ARCH.ident(),
                dtb1: dtb,
                dtb2: Address::invalid(),
            },
            dtb,
            map_size,
            modules: vec![],
        }
    }

    pub fn alloc_process(&mut self, map_size: usize, test_buf: &[u8]) -> Pid {
        let proc = self.internal_alloc_process(map_size, test_buf);

        let ret = proc.info.pid;

        self.processes.push(proc);

        ret
    }

    pub fn alloc_process_with_module(&mut self, map_size: usize, test_buf: &[u8]) -> Pid {
        let mut proc = self.internal_alloc_process(map_size, test_buf);

        let ret = proc.info.pid;

        proc.add_modules(1, map_size / 2);

        self.processes.push(proc);

        ret
    }

    pub fn alloc_dtb(&mut self, map_size: usize, test_buf: &[u8]) -> (Address, Address) {
        let virt_base = (Address::null()
            + self
                .rng
                .gen_range(0x0001_0000_0000_u64..((!0_u64) << 20) >> 20))
        .as_mem_aligned(mem::gb(2));

        (
            self.alloc_dtb_const_base(virt_base, map_size, test_buf),
            virt_base,
        )
    }

    pub fn process_alloc_random_mem(&mut self, proc: &DummyProcessInfo, cnt: usize, size: usize) {
        for _ in 0..cnt {
            let virt_base = (Address::null()
                + self
                    .rng
                    .gen_range(0x0001_0000_0000_u64..((!0_u64) << 20) >> 20))
            .as_mem_aligned(mem::gb(2));

            self.alloc_mem_to_dtb(proc.dtb, virt_base, size, &[]);
        }
    }

    pub fn alloc_dtb_const_base(
        &mut self,
        virt_base: Address,
        map_size: usize,
        test_buf: &[u8],
    ) -> Address {
        let dtb = self.alloc_pt_page().addr;

        unsafe {
            *(self
                .mem
                .buf_ptr()
                .add(clamp_to_usize(dtb.to_umem()))
                .cast::<PageTable>() as *mut _) = PageTable::new()
        };

        self.alloc_mem_to_dtb(dtb, virt_base, map_size, test_buf)
    }

    pub fn alloc_mem_to_dtb(
        &mut self,
        dtb: Address,
        virt_base: Address,
        map_size: usize,
        test_buf: &[u8],
    ) -> Address {
        let mut cur_len = 0;

        let pml4 = unsafe {
            &mut *(self
                .mem
                .buf_ptr()
                .add(clamp_to_usize(dtb.to_umem()))
                .cast::<PageTable>() as *mut _)
        };

        let mut pt_mapper =
            unsafe { OffsetPageTable::new(pml4, VirtAddr::from_ptr(self.mem.buf_ptr())) };

        while cur_len < map_size {
            let page_info = self.next_page_for_address(cur_len.into());
            let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;

            if test_buf.len() >= (cur_len + page_info.size.to_size() as usize) {
                self.mem
                    .phys_write(
                        page_info.addr.into(),
                        &test_buf[cur_len..(cur_len + page_info.size.to_size() as usize)],
                    )
                    .unwrap();
            } else if test_buf.len() > cur_len {
                self.mem
                    .phys_write(page_info.addr.into(), &test_buf[cur_len..])
                    .unwrap();
            }

            unsafe {
                match page_info.size {
                    X64PageSize::P1g => pt_mapper
                        .map_to(
                            paging::page::Page::<Size1GiB>::from_start_address_unchecked(
                                VirtAddr::new((virt_base + cur_len).to_umem() as u64),
                            ),
                            PhysFrame::from_start_address_unchecked(PhysAddr::new(
                                page_info.addr.to_umem() as u64,
                            )),
                            flags | PageTableFlags::HUGE_PAGE,
                            self,
                        )
                        .is_ok(),
                    X64PageSize::P2m => pt_mapper
                        .map_to(
                            paging::page::Page::<Size2MiB>::from_start_address_unchecked(
                                VirtAddr::new((virt_base + cur_len).to_umem() as u64),
                            ),
                            PhysFrame::from_start_address_unchecked(PhysAddr::new(
                                page_info.addr.to_umem() as u64,
                            )),
                            flags | PageTableFlags::HUGE_PAGE,
                            self,
                        )
                        .is_ok(),
                    X64PageSize::P4k => pt_mapper
                        .map_to(
                            paging::page::Page::<Size4KiB>::from_start_address_unchecked(
                                VirtAddr::new((virt_base + cur_len).to_umem() as u64),
                            ),
                            PhysFrame::from_start_address_unchecked(PhysAddr::new(
                                page_info.addr.to_umem() as u64,
                            )),
                            flags,
                            self,
                        )
                        .is_ok(),
                };
            }
            cur_len += page_info.size.to_size();
        }

        dtb
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

pub type DummyVirtMem<T> = VirtualDma<T, DirectTranslate, X86VirtualTranslate>;

impl Os for DummyOs {
    type ProcessType<'a> = DummyProcess<DummyVirtMem<Fwd<&'a mut DummyMemory>>>;
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
            .ok_or(Error(ErrorOrigin::OsLayer, ErrorKind::ProcessNotFound))
            .map(|p| p.info.clone())
    }

    /// Creates a process by its internal address
    ///
    /// It will share the underlying memory resources
    fn process_by_info(&mut self, info: ProcessInfo) -> Result<Self::ProcessType<'_>> {
        let proc = self
            .processes
            .iter()
            .find(|p| p.info.address == info.address)
            .ok_or(Error(ErrorOrigin::OsLayer, ErrorKind::InvalidProcessInfo))?
            .clone();
        Ok(DummyProcess {
            mem: VirtualDma::new(
                self.mem.forward_mut(),
                x64::ARCH,
                x64::new_translator(proc.dtb),
            ),
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
            .ok_or(Error(ErrorOrigin::OsLayer, ErrorKind::InvalidProcessInfo))?
            .clone();
        Ok(DummyProcess {
            mem: VirtualDma::new(self.mem, x64::ARCH, x64::new_translator(proc.dtb)),
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
        Err(Error(ErrorOrigin::OsLayer, ErrorKind::ModuleNotFound))
    }

    /// Retrieves address of the primary module structure of the process
    ///
    /// This will generally be for the initial executable that was run
    fn primary_module_address(&mut self) -> Result<Address> {
        Err(Error(ErrorOrigin::OsLayer, ErrorKind::ModuleNotFound))
    }

    /// Retrieves a list of all imports of a given module
    fn module_import_list_callback(
        &mut self,
        _info: &ModuleInfo,
        _callback: ImportCallback,
    ) -> Result<()> {
        Ok(())
    }

    /// Retrieves a list of all exports of a given module
    fn module_export_list_callback(
        &mut self,
        _info: &ModuleInfo,
        _callback: ExportCallback,
    ) -> Result<()> {
        Ok(())
    }

    /// Retrieves a list of all sections of a given module
    fn module_section_list_callback(
        &mut self,
        _info: &ModuleInfo,
        _callback: SectionCallback,
    ) -> Result<()> {
        Ok(())
    }

    /// Retrieves the kernel info
    fn info(&self) -> &OsInfo {
        &self.info
    }
}

impl PhysicalMemory for DummyOs {
    #[inline]
    fn phys_read_raw_iter(&mut self, data: PhysicalReadMemOps) -> Result<()> {
        self.mem.phys_read_raw_iter(data)
    }

    #[inline]
    fn phys_write_raw_iter(&mut self, data: PhysicalWriteMemOps) -> Result<()> {
        self.mem.phys_write_raw_iter(data)
    }

    #[inline]
    fn metadata(&self) -> PhysicalMemoryMetadata {
        self.mem.metadata()
    }

    #[inline]
    fn set_mem_map(&mut self, mem_map: &[PhysicalMemoryMapping]) {
        self.mem.set_mem_map(mem_map)
    }
}

#[doc(hidden)]
#[no_mangle]
pub static MEMFLOW_OS_DUMMY: OsDescriptor = OsDescriptor {
    plugin_version: MEMFLOW_PLUGIN_VERSION,
    accept_input: false,
    input_layout: <<LoadableOs as Loadable>::CInputArg as ::abi_stable::StableAbi>::LAYOUT,
    output_layout: <<LoadableOs as Loadable>::Instance as ::abi_stable::StableAbi>::LAYOUT,
    name: CSliceRef::from_str("dummy"),
    version: CSliceRef::from_str(env!("CARGO_PKG_VERSION")),
    description: CSliceRef::from_str("Dummy testing OS"),
    help_callback: None, // TODO: add dummy help string
    target_list_callback: None,
    create: mf_create,
};

#[doc(hidden)]
extern "C" fn mf_create(
    args: Option<&OsArgs>,
    _connector: COption<ConnectorInstanceArcBox>,
    lib: LibArc,
    logger: Option<&'static PluginLogger>,
    out: &mut MuOsInstanceArcBox<'static>,
) -> i32 {
    plugins::wrap(args, lib, logger, out, create_dummy)
}

pub fn create_dummy(args: &OsArgs, lib: LibArc) -> Result<OsInstanceArcBox<'static>> {
    let size = super::mem::parse_size(&args.extra_args)?;
    let mem = DummyMemory::new(size);
    let mut os = DummyOs::new(mem);
    os.alloc_process_with_module(
        std::cmp::min(
            size::mb(2),
            size.saturating_sub(size::mb(2)) + size::kb(512),
        ),
        &[],
    );
    let os = CBox::from(os);
    let obj = group_obj!((os, lib) as OsInstance);
    Ok(obj)
    // Err(Error(
    //     ErrorOrigin::Connector,
    //     ErrorKind::InvalidMemorySizeUnit,
    // ))
}
