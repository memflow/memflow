use memflow_core::mem::virt_mem::*;
use memflow_core::types::Address;

use crate::util::*;

use std::slice::{from_raw_parts, from_raw_parts_mut};

pub type VirtualMemoryObj = &'static mut dyn VirtualMemory;

#[no_mangle]
pub unsafe extern "C" fn virt_read_raw_list(
    mem: &mut VirtualMemoryObj,
    data: *mut VirtualReadData,
    len: usize,
) -> i32 {
    let data = from_raw_parts_mut(data, len);
    mem.virt_read_raw_list(data).int_result()
}

#[no_mangle]
pub unsafe extern "C" fn virt_write_raw_list(
    mem: &mut VirtualMemoryObj,
    data: *const VirtualWriteData,
    len: usize,
) -> i32 {
    let data = from_raw_parts(data, len);
    mem.virt_write_raw_list(data).int_result()
}

#[no_mangle]
pub unsafe extern "C" fn virt_read_raw(
    mem: &mut VirtualMemoryObj,
    addr: Address,
    out: *mut u8,
    len: usize,
) -> i32 {
    mem.virt_read_raw_into(addr, from_raw_parts_mut(out, len))
        .int_result()
}

#[no_mangle]
pub extern "C" fn virt_read_u32(mem: &mut VirtualMemoryObj, addr: Address) -> u32 {
    mem.virt_read::<u32>(addr).unwrap_or_default()
}

#[no_mangle]
pub extern "C" fn virt_read_u64(mem: &mut VirtualMemoryObj, addr: Address) -> u64 {
    mem.virt_read::<u64>(addr).unwrap_or_default()
}

#[no_mangle]
pub unsafe extern "C" fn virt_write_raw(
    mem: &mut VirtualMemoryObj,
    addr: Address,
    input: *const u8,
    len: usize,
) -> i32 {
    mem.virt_write_raw(addr, from_raw_parts(input, len))
        .int_result()
}

#[no_mangle]
pub extern "C" fn virt_write_u32(mem: &mut VirtualMemoryObj, addr: Address, val: u32) -> i32 {
    mem.virt_write(addr, &val).int_result()
}

#[no_mangle]
pub extern "C" fn virt_write_u64(mem: &mut VirtualMemoryObj, addr: Address, val: u64) -> i32 {
    mem.virt_write(addr, &val).int_result()
}
