use memflow_core::mem::phys_mem::*;
use memflow_core::types::PhysicalAddress;

use crate::util::*;

use std::slice::{from_raw_parts, from_raw_parts_mut};

pub type CloneablePhysicalMemoryObj = &'static mut dyn CloneablePhysicalMemory;
pub type PhysicalMemoryObj = &'static mut dyn CloneablePhysicalMemory;

#[no_mangle]
pub unsafe extern "C" fn downcast_cloneable(
    cloneable: &'static mut CloneablePhysicalMemoryObj,
) -> &'static PhysicalMemoryObj {
    Box::leak(Box::from_raw(cloneable))
}

#[no_mangle]
pub unsafe extern "C" fn phys_read_raw_list(
    mem: &mut PhysicalMemoryObj,
    data: *mut PhysicalReadData,
    len: usize,
) -> i32 {
    let data = from_raw_parts_mut(data, len);
    mem.phys_read_raw_list(data).int_result()
}

#[no_mangle]
pub unsafe extern "C" fn phys_write_raw_list(
    mem: &mut PhysicalMemoryObj,
    data: *const PhysicalWriteData,
    len: usize,
) -> i32 {
    let data = from_raw_parts(data, len);
    mem.phys_write_raw_list(data).int_result()
}

#[no_mangle]
pub unsafe extern "C" fn phys_metadata(mem: &mut PhysicalMemoryObj) -> PhysicalMemoryMetadata {
    mem.metadata()
}

#[no_mangle]
pub unsafe extern "C" fn phys_read_raw_into(
    mem: &mut PhysicalMemoryObj,
    addr: PhysicalAddress,
    out: *mut u8,
    len: usize,
) -> i32 {
    mem.phys_read_raw_into(addr, from_raw_parts_mut(out, len))
        .int_result()
}
