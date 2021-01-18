use super::util::*;
use crate::error::{Error, PartialResult, Result};
use crate::mem::{VirtualMemory, VirtualReadData, VirtualWriteData};
use crate::types::Address;
use crate::types::{Page, PhysicalAddress};
use std::ffi::c_void;

use super::util::result_from_int_void;

pub type OpaqueVirtualMemoryFunctionTable = VirtualMemoryFunctionTable<c_void>;

#[repr(C)]
#[derive(Clone, Copy)]
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
}

impl<T: VirtualMemory + Sized> VirtualMemoryFunctionTable<T> {
    pub fn into_opaque(self) -> OpaqueVirtualMemoryFunctionTable {
        unsafe { std::mem::transmute(self) }
    }
}

impl<T: VirtualMemory> Default for VirtualMemoryFunctionTable<T> {
    fn default() -> Self {
        Self {
            virt_read_raw_list: c_virt_read_raw_list,
            virt_write_raw_list: c_virt_write_raw_list,
        }
    }
}

extern "C" fn c_virt_read_raw_list<T: VirtualMemory>(
    virt_mem: &mut T,
    read_data: *mut VirtualReadData,
    read_data_count: usize,
) -> i32 {
    let read_data_slice = unsafe { std::slice::from_raw_parts_mut(read_data, read_data_count) };
    virt_mem.virt_read_raw_list(read_data_slice).int_result()
}

extern "C" fn c_virt_write_raw_list<T: VirtualMemory>(
    virt_mem: &mut T,
    write_data: *const VirtualWriteData,
    write_data_count: usize,
) -> i32 {
    let write_data_slice = unsafe { std::slice::from_raw_parts(write_data, write_data_count) };
    virt_mem.virt_write_raw_list(write_data_slice).int_result()
}

#[repr(C)]
pub struct VirtualMemoryInstance<'a> {
    instance: &'a mut c_void,
    vtable: OpaqueVirtualMemoryFunctionTable,
}

impl<'a> VirtualMemoryInstance<'a> {
    pub fn new<T: VirtualMemory>(instance: &'a mut T) -> Self {
        Self {
            instance: unsafe { (instance as *mut T as *mut c_void).as_mut() }.unwrap(),
            vtable: VirtualMemoryFunctionTable::<T>::default().into_opaque(),
        }
    }

    pub unsafe fn unsafe_new<T: VirtualMemory>(instance: &'a mut c_void) -> Self {
        Self {
            instance,
            vtable: VirtualMemoryFunctionTable::<T>::default().into_opaque(),
        }
    }
}

impl VirtualMemoryInstance<'static> {
    fn leak_new<T: VirtualMemory + 'static>(mem: T) -> Self {
        Self::new(Box::leak(mem.into()))
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
        Err(Error::Other("unimplemented"))
    }

    fn virt_translation_map_range(
        &mut self,
        start: Address,
        end: Address,
    ) -> Vec<(Address, usize, PhysicalAddress)> {
        vec![]
    }

    fn virt_page_map_range(
        &mut self,
        gap_size: usize,
        start: Address,
        end: Address,
    ) -> Vec<(Address, usize)> {
        vec![]
    }
}
