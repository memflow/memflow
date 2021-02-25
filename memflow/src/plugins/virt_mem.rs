use crate::error::*;
use crate::mem::{VirtualMemory, VirtualReadData, VirtualWriteData};
use crate::types::Address;
use crate::types::{OpaqueCallback, Page, PhysicalAddress};
use std::ffi::c_void;

pub type OpaqueVirtualMemoryFunctionTable = VirtualMemoryFunctionTable<c_void>;
pub type MUPage = std::mem::MaybeUninit<Page>;

impl Copy for OpaqueVirtualMemoryFunctionTable {}

impl Clone for OpaqueVirtualMemoryFunctionTable {
    fn clone(&self) -> Self {
        *self
    }
}

#[repr(C)]
pub struct TranslationChunk(pub Address, pub usize, pub PhysicalAddress);

pub type TranslationMapCallback<'a> = OpaqueCallback<'a, TranslationChunk>;

#[repr(C)]
pub struct PageMapChunk(pub Address, pub usize);

pub type PageMapCallback<'a> = OpaqueCallback<'a, PageMapChunk>;

#[repr(C)]
pub struct VirtualMemoryFunctionTable<T> {
    pub virt_read_raw_list: extern "C" fn(
        virt_mem: &mut T,
        read_data: *mut VirtualReadData,
        read_data_count: usize,
    ) -> i32,
    pub virt_write_raw_list: extern "C" fn(
        virt_mem: &mut T,
        write_data: *const VirtualWriteData,
        write_data_count: usize,
    ) -> i32,
    pub virt_page_info: extern "C" fn(virt_mem: &mut T, addr: Address, out: &mut MUPage) -> i32,
    pub virt_translation_map_range:
        extern "C" fn(virt_mem: &mut T, start: Address, end: Address, out: TranslationMapCallback),
    pub virt_page_map_range: extern "C" fn(
        virt_mem: &mut T,
        gap_size: usize,
        start: Address,
        end: Address,
        out: PageMapCallback,
    ),
}

impl<T: VirtualMemory + Sized> VirtualMemoryFunctionTable<T> {
    pub fn as_opaque(&self) -> &OpaqueVirtualMemoryFunctionTable {
        unsafe { &*(self as *const Self as *const OpaqueVirtualMemoryFunctionTable) }
    }
}

impl<'a, T: VirtualMemory> Default for &'a VirtualMemoryFunctionTable<T> {
    fn default() -> Self {
        &VirtualMemoryFunctionTable {
            virt_read_raw_list: c_virt_read_raw_list,
            virt_write_raw_list: c_virt_write_raw_list,
            virt_page_info: c_virt_page_info,
            virt_translation_map_range: c_virt_translation_map_range,
            virt_page_map_range: c_virt_page_map_range,
        }
    }
}

extern "C" fn c_virt_read_raw_list<T: VirtualMemory>(
    virt_mem: &mut T,
    read_data: *mut VirtualReadData,
    read_data_count: usize,
) -> i32 {
    let read_data_slice = unsafe { std::slice::from_raw_parts_mut(read_data, read_data_count) };
    virt_mem.virt_read_raw_list(read_data_slice).as_int_result()
}

extern "C" fn c_virt_write_raw_list<T: VirtualMemory>(
    virt_mem: &mut T,
    write_data: *const VirtualWriteData,
    write_data_count: usize,
) -> i32 {
    let write_data_slice = unsafe { std::slice::from_raw_parts(write_data, write_data_count) };
    virt_mem
        .virt_write_raw_list(write_data_slice)
        .as_int_result()
}

extern "C" fn c_virt_page_info<T: VirtualMemory>(
    virt_mem: &mut T,
    addr: Address,
    out: &mut MUPage,
) -> i32 {
    virt_mem.virt_page_info(addr).as_int_out_result(out)
}

extern "C" fn c_virt_translation_map_range<T: VirtualMemory>(
    virt_mem: &mut T,
    start: Address,
    end: Address,
    mut out: TranslationMapCallback,
) {
    let vec = virt_mem.virt_translation_map_range(start, end);
    for (a, b, c) in vec {
        if !out.call(TranslationChunk(a, b, c)) {
            break;
        }
    }
}

extern "C" fn c_virt_page_map_range<T: VirtualMemory>(
    virt_mem: &mut T,
    gap_size: usize,
    start: Address,
    end: Address,
    mut out: PageMapCallback,
) {
    let vec = virt_mem.virt_page_map_range(gap_size, start, end);
    for (a, b) in vec {
        if !out.call(PageMapChunk(a, b)) {
            break;
        }
    }
}

#[repr(C)]
pub struct VirtualMemoryInstance<'a> {
    pub(crate) instance: &'a mut c_void,
    pub(crate) vtable: &'a OpaqueVirtualMemoryFunctionTable,
}

impl<'a> VirtualMemoryInstance<'a> {
    pub fn new<T: 'a + VirtualMemory>(instance: &'a mut T) -> Self {
        Self {
            instance: unsafe { (instance as *mut T as *mut c_void).as_mut() }.unwrap(),
            vtable: <&VirtualMemoryFunctionTable<T>>::default().as_opaque(),
        }
    }

    /// # Safety
    ///
    /// The type of `instance` has to match T
    pub unsafe fn unsafe_new<T: 'a + VirtualMemory>(instance: &'a mut c_void) -> Self {
        Self {
            instance,
            vtable: <&VirtualMemoryFunctionTable<T>>::default().as_opaque(),
        }
    }
}

impl VirtualMemory for VirtualMemoryInstance<'_> {
    fn virt_read_raw_list(&mut self, data: &mut [VirtualReadData]) -> PartialResult<()> {
        let res = (self.vtable.virt_read_raw_list)(self.instance, data.as_mut_ptr(), data.len());
        part_result_from_int_void(res)
    }

    fn virt_write_raw_list(&mut self, data: &[VirtualWriteData]) -> PartialResult<()> {
        let res = (self.vtable.virt_write_raw_list)(self.instance, data.as_ptr(), data.len());
        part_result_from_int_void(res)
    }

    fn virt_page_info(&mut self, addr: Address) -> Result<Page> {
        let mut out = MUPage::uninit();
        let res = (self.vtable.virt_page_info)(self.instance, addr, &mut out);
        result_from_int(res, out)
    }

    fn virt_translation_map_range(
        &mut self,
        start: Address,
        end: Address,
    ) -> Vec<(Address, usize, PhysicalAddress)> {
        let mut ret = vec![];
        let f = &mut |TranslationChunk(a, b, c)| {
            ret.push((a, b, c));
            true
        };
        (self.vtable.virt_translation_map_range)(self.instance, start, end, f.into());
        ret
    }

    fn virt_page_map_range(
        &mut self,
        gap_size: usize,
        start: Address,
        end: Address,
    ) -> Vec<(Address, usize)> {
        let mut ret = vec![];
        let f = &mut |PageMapChunk(a, b)| {
            ret.push((a, b));
            true
        };
        (self.vtable.virt_page_map_range)(self.instance, gap_size, start, end, f.into());
        ret
    }
}
