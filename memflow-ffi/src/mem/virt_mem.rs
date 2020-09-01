use memflow::error::PartialResultExt;
use memflow::mem::virt_mem::*;
use memflow::types::Address;

use crate::util::*;

use std::slice::{from_raw_parts, from_raw_parts_mut};

pub type VirtualMemoryObj = &'static mut dyn VirtualMemory;

/// Free a virtual memory object reference
///
/// This function frees the reference to a virtual memory object.
///
/// # Safety
///
/// `mem` must be a valid reference to a virtual memory object.
#[no_mangle]
pub unsafe extern "C" fn virt_free(mem: &'static mut VirtualMemoryObj) {
    let _ = Box::from_raw(mem);
}

/// Read a list of values
///
/// This will perform `len` virtual memory reads on the provided `data`. Using lists is preferable
/// for performance, because then the underlying connectors can batch those operations, and virtual
/// translation function can cut down on read operations.
///
/// # Safety
///
/// `data` must be a valid array of `VirtualReadData` with the length of at least `len`
#[no_mangle]
pub unsafe extern "C" fn virt_read_raw_list(
    mem: &mut VirtualMemoryObj,
    data: *mut VirtualReadData,
    len: usize,
) -> i32 {
    let data = from_raw_parts_mut(data, len);
    mem.virt_read_raw_list(data).data_part().int_result()
}

/// Write a list of values
///
/// This will perform `len` virtual memory writes on the provided `data`. Using lists is preferable
/// for performance, because then the underlying connectors can batch those operations, and virtual
/// translation function can cut down on read operations.
///
/// # Safety
///
/// `data` must be a valid array of `VirtualWriteData` with the length of at least `len`
#[no_mangle]
pub unsafe extern "C" fn virt_write_raw_list(
    mem: &mut VirtualMemoryObj,
    data: *const VirtualWriteData,
    len: usize,
) -> i32 {
    let data = from_raw_parts(data, len);
    mem.virt_write_raw_list(data).data_part().int_result()
}

/// Read a single value into `out` from a provided `Address`
///
/// # Safety
///
/// `out` must be a valid pointer to a data buffer of at least `len` size.
#[no_mangle]
pub unsafe extern "C" fn virt_read_raw_into(
    mem: &mut VirtualMemoryObj,
    addr: Address,
    out: *mut u8,
    len: usize,
) -> i32 {
    mem.virt_read_raw_into(addr, from_raw_parts_mut(out, len))
        .data_part()
        .int_result()
}

/// Read a single 32-bit value from a provided `Address`
#[no_mangle]
pub extern "C" fn virt_read_u32(mem: &mut VirtualMemoryObj, addr: Address) -> u32 {
    mem.virt_read::<u32>(addr).unwrap_or_default()
}

/// Read a single 64-bit value from a provided `Address`
#[no_mangle]
pub extern "C" fn virt_read_u64(mem: &mut VirtualMemoryObj, addr: Address) -> u64 {
    mem.virt_read::<u64>(addr).unwrap_or_default()
}

/// Write a single value from `input` into a provided `Address`
///
/// # Safety
///
/// `input` must be a valid pointer to a data buffer of at least `len` size.
#[no_mangle]
pub unsafe extern "C" fn virt_write_raw(
    mem: &mut VirtualMemoryObj,
    addr: Address,
    input: *const u8,
    len: usize,
) -> i32 {
    mem.virt_write_raw(addr, from_raw_parts(input, len))
        .data_part()
        .int_result()
}

/// Write a single 32-bit value into a provided `Address`
#[no_mangle]
pub extern "C" fn virt_write_u32(mem: &mut VirtualMemoryObj, addr: Address, val: u32) -> i32 {
    mem.virt_write(addr, &val).data_part().int_result()
}

/// Write a single 64-bit value into a provided `Address`
#[no_mangle]
pub extern "C" fn virt_write_u64(mem: &mut VirtualMemoryObj, addr: Address, val: u64) -> i32 {
    mem.virt_write(addr, &val).data_part().int_result()
}
