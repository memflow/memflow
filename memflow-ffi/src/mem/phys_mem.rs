use memflow_core::mem::phys_mem::*;
use memflow_core::types::{Address, PhysicalAddress};

use crate::util::*;

use std::slice::{from_raw_parts, from_raw_parts_mut};

use log::trace;

pub type CloneablePhysicalMemoryObj = &'static mut dyn CloneablePhysicalMemory;
pub type PhysicalMemoryObj = &'static mut dyn PhysicalMemory;

#[no_mangle]
pub unsafe extern "C" fn downcast_cloneable(
    cloneable: &'static mut CloneablePhysicalMemoryObj,
) -> &'static mut PhysicalMemoryObj {
    Box::leak(Box::new((*cloneable).downcast()))
}

#[no_mangle]
pub unsafe extern "C" fn phys_free(mem: &'static mut PhysicalMemoryObj) {
    trace!("phys_free: {:?}", mem as *mut _);
    let _ = Box::from_raw(mem);
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
pub extern "C" fn phys_metadata(mem: &mut PhysicalMemoryObj) -> PhysicalMemoryMetadata {
    mem.metadata()
}

#[no_mangle]
pub unsafe extern "C" fn phys_read_raw(
    mem: &mut PhysicalMemoryObj,
    addr: PhysicalAddress,
    out: *mut u8,
    len: usize,
) -> i32 {
    mem.phys_read_raw_into(addr, from_raw_parts_mut(out, len))
        .int_result()
}

#[no_mangle]
pub extern "C" fn phys_read_u32(mem: &mut PhysicalMemoryObj, addr: PhysicalAddress) -> u32 {
    mem.phys_read::<u32>(addr).unwrap_or_default()
}

#[no_mangle]
pub extern "C" fn phys_read_u64(mem: &mut PhysicalMemoryObj, addr: PhysicalAddress) -> u64 {
    mem.phys_read::<u64>(addr).unwrap_or_default()
}

#[no_mangle]
pub unsafe extern "C" fn phys_write_raw(
    mem: &mut PhysicalMemoryObj,
    addr: PhysicalAddress,
    input: *const u8,
    len: usize,
) -> i32 {
    mem.phys_write_raw(addr, from_raw_parts(input, len))
        .int_result()
}

#[no_mangle]
pub extern "C" fn phys_write_u32(
    mem: &mut PhysicalMemoryObj,
    addr: PhysicalAddress,
    val: u32,
) -> i32 {
    mem.phys_write(addr, &val).int_result()
}

#[no_mangle]
pub extern "C" fn phys_write_u64(
    mem: &mut PhysicalMemoryObj,
    addr: PhysicalAddress,
    val: u64,
) -> i32 {
    mem.phys_write(addr, &val).int_result()
}
