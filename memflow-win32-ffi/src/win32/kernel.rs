use memflow_ffi::mem::phys_mem::CloneablePhysicalMemoryObj;
use memflow_ffi::util::*;
use memflow_win32::kernel::Win32Version;
use memflow_win32::win32::{kernel, Win32ProcessInfo, Win32VirtualTranslate};

use memflow::mem::{
    cache::{CachedMemoryAccess, CachedVirtualTranslate, TimedCacheValidator},
    CloneablePhysicalMemory, DirectTranslate, VirtualDMA,
};

use memflow::iter::FnExtend;
use memflow::process::PID;
use memflow::types::{size, Address, PageType};

use super::process::Win32Process;
use crate::kernel::start_block::StartBlock;

use std::ffi::CStr;
use std::os::raw::c_char;
use std::time::Duration;

pub(crate) type FFIMemory =
    CachedMemoryAccess<'static, Box<dyn CloneablePhysicalMemory>, TimedCacheValidator>;
pub(crate) type FFIVirtualTranslate = CachedVirtualTranslate<DirectTranslate, TimedCacheValidator>;

pub(crate) type FFIVirtualMemory =
    VirtualDMA<FFIMemory, FFIVirtualTranslate, Win32VirtualTranslate>;

pub type Kernel = kernel::Kernel<FFIMemory, FFIVirtualTranslate>;

/// Build a cloneable kernel object with default caching parameters
///
/// This function will take ownership of the input `mem` object.
///
/// # Safety
///
/// `mem` must be a heap allocated memory reference, created by one of the API's functions.
/// Reference to it becomes invalid.
#[no_mangle]
pub unsafe extern "C" fn kernel_build(
    mem: &'static mut CloneablePhysicalMemoryObj,
) -> Option<&'static mut Kernel> {
    let mem: Box<dyn CloneablePhysicalMemory> = Box::from_raw(*Box::from_raw(mem));
    kernel::Kernel::builder(mem)
        .build_default_caches()
        .build()
        .map_err(inspect_err)
        .ok()
        .map(to_heap)
}

/// Build a cloneable kernel object with custom caching parameters
///
/// This function will take ownership of the input `mem` object.
///
/// vat_cache_entries must be positive, or the program will panic upon memory reads or writes.
///
/// # Safety
///
/// `mem` must be a heap allocated memory reference, created by one of the API's functions.
/// Reference to it becomes invalid.
#[no_mangle]
pub unsafe extern "C" fn kernel_build_custom(
    mem: &'static mut CloneablePhysicalMemoryObj,
    page_cache_time_ms: u64,
    page_cache_flags: PageType,
    page_cache_size_kb: usize,
    vat_cache_time_ms: u64,
    vat_cache_entries: usize,
) -> Option<&'static mut Kernel> {
    let mem: Box<dyn CloneablePhysicalMemory> = Box::from_raw(*Box::from_raw(mem));
    kernel::Kernel::builder(mem)
        .build_page_cache(move |connector, arch| {
            CachedMemoryAccess::builder(connector)
                .arch(arch)
                .validator(TimedCacheValidator::new(
                    Duration::from_millis(page_cache_time_ms).into(),
                ))
                .page_type_mask(page_cache_flags)
                .cache_size(size::kb(page_cache_size_kb))
                .build()
                .unwrap()
        })
        .build_vat_cache(move |vat, arch| {
            CachedVirtualTranslate::builder(vat)
                .arch(arch)
                .validator(TimedCacheValidator::new(
                    Duration::from_millis(vat_cache_time_ms).into(),
                ))
                .entries(vat_cache_entries)
                .build()
                .unwrap()
        })
        .build()
        .map_err(inspect_err)
        .ok()
        .map(to_heap)
}

#[no_mangle]
pub extern "C" fn kernel_clone(kernel: &Kernel) -> &'static mut Kernel {
    Box::leak(Box::new((*kernel).clone()))
}

/// Free a kernel object
///
/// This will free the input `kernel` object (including the underlying memory object)
///
/// # Safety
///
/// `kernel` must be a valid reference heap allocated by one of the above functions.
#[no_mangle]
pub unsafe extern "C" fn kernel_free(kernel: &'static mut Kernel) {
    let _ = Box::from_raw(kernel);
}

/// Destroy a kernel object and return its underlying memory object
///
/// This will free the input `kernel` object, and return the underlying memory object. It will free
/// the object from any additional caching that `kernel` had in place.
///
/// # Safety
///
/// `kernel` must be a valid reference heap allocated by one of the above functions.
#[no_mangle]
pub unsafe extern "C" fn kernel_destroy(
    kernel: &'static mut Kernel,
) -> &'static mut CloneablePhysicalMemoryObj {
    let kernel = Box::from_raw(kernel);
    Box::leak(Box::new(Box::leak(kernel.destroy().destroy())))
}

#[no_mangle]
pub extern "C" fn kernel_start_block(kernel: &Kernel) -> StartBlock {
    kernel.kernel_info.start_block.into()
}

#[no_mangle]
pub extern "C" fn kernel_winver(kernel: &Kernel) -> Win32Version {
    kernel.kernel_info.kernel_winver.mask_build_number()
}

#[no_mangle]
pub extern "C" fn kernel_winver_unmasked(kernel: &Kernel) -> Win32Version {
    kernel.kernel_info.kernel_winver
}

/// Retrieve a list of peorcess addresses
///
/// # Safety
///
/// `buffer` must be a valid buffer of size at least `max_size`
#[no_mangle]
pub unsafe extern "C" fn kernel_eprocess_list(
    kernel: &'static mut Kernel,
    buffer: *mut Address,
    max_size: usize,
) -> usize {
    let mut ret = 0;

    let buffer = std::slice::from_raw_parts_mut(buffer, max_size);

    let mut extend_fn = FnExtend::new(|addr| {
        if ret < max_size {
            buffer[ret] = addr;
            ret += 1;
        }
    });

    kernel
        .eprocess_list_extend(&mut extend_fn)
        .map_err(inspect_err)
        .ok()
        .map(|_| ret)
        .unwrap_or_default()
}

/// Retrieve a list of processes
///
/// This will fill `buffer` with a list of win32 process information. These processes will need to be
/// individually freed with `process_info_free`
///
/// # Safety
///
/// `buffer` must be a valid that can contain at least `max_size` references to `Win32ProcessInfo`.
#[no_mangle]
pub unsafe extern "C" fn kernel_process_info_list(
    kernel: &'static mut Kernel,
    buffer: *mut &'static mut Win32ProcessInfo,
    max_size: usize,
) -> usize {
    let mut ret = 0;

    let buffer = std::slice::from_raw_parts_mut(buffer, max_size);

    let mut extend_fn = FnExtend::new(|info| {
        if ret < max_size {
            buffer[ret] = Box::leak(Box::new(info));
            ret += 1;
        }
    });

    kernel
        .process_info_list_extend(&mut extend_fn)
        .map_err(inspect_err)
        .ok()
        .map(|_| ret)
        .unwrap_or_default()
}

// Process info

#[no_mangle]
pub extern "C" fn kernel_kernel_process_info(
    kernel: &'static mut Kernel,
) -> Option<&'static mut Win32ProcessInfo> {
    kernel
        .kernel_process_info()
        .map_err(inspect_err)
        .ok()
        .map(to_heap)
}

#[no_mangle]
pub extern "C" fn kernel_process_info_from_eprocess(
    kernel: &'static mut Kernel,
    eprocess: Address,
) -> Option<&'static mut Win32ProcessInfo> {
    kernel
        .process_info_from_eprocess(eprocess)
        .map_err(inspect_err)
        .ok()
        .map(to_heap)
}

/// Retrieve process information by name
///
/// # Safety
///
/// `name` must be a valid null terminated string
#[no_mangle]
pub unsafe extern "C" fn kernel_process_info(
    kernel: &'static mut Kernel,
    name: *const c_char,
) -> Option<&'static mut Win32ProcessInfo> {
    let name = CStr::from_ptr(name).to_string_lossy();
    kernel
        .process_info(&name)
        .map_err(inspect_err)
        .ok()
        .map(to_heap)
}

#[no_mangle]
pub extern "C" fn kernel_process_info_pid(
    kernel: &'static mut Kernel,
    pid: PID,
) -> Option<&'static mut Win32ProcessInfo> {
    kernel
        .process_info_pid(pid)
        .map_err(inspect_err)
        .ok()
        .map(to_heap)
}

// Process conversion

/// Create a process by looking up its name
///
/// This will consume `kernel` and free it later on.
///
/// # Safety
///
/// `name` must be a valid null terminated string
///
/// `kernel` must be a valid reference to `Kernel`. After the function the reference to it becomes
/// invalid.
#[no_mangle]
pub unsafe extern "C" fn kernel_into_process(
    kernel: &'static mut Kernel,
    name: *const c_char,
) -> Option<&'static mut Win32Process> {
    let kernel = Box::from_raw(kernel);
    let name = CStr::from_ptr(name).to_string_lossy();
    kernel
        .into_process(&name)
        .map_err(inspect_err)
        .ok()
        .map(to_heap)
}

/// Create a process by looking up its PID
///
/// This will consume `kernel` and free it later on.
///
/// # Safety
///
/// `kernel` must be a valid reference to `Kernel`. After the function the reference to it becomes
/// invalid.
#[no_mangle]
pub unsafe extern "C" fn kernel_into_process_pid(
    kernel: &'static mut Kernel,
    pid: PID,
) -> Option<&'static mut Win32Process> {
    let kernel = Box::from_raw(kernel);
    kernel
        .into_process_pid(pid)
        .map_err(inspect_err)
        .ok()
        .map(to_heap)
}

/// Create a kernel process insatance
///
/// This will consume `kernel` and free it later on.
///
/// # Safety
///
/// `kernel` must be a valid reference to `Kernel`. After the function the reference to it becomes
/// invalid.
#[no_mangle]
pub unsafe extern "C" fn kernel_into_kernel_process(
    kernel: &'static mut Kernel,
) -> Option<&'static mut Win32Process> {
    let kernel = Box::from_raw(kernel);
    kernel
        .into_kernel_process()
        .map_err(inspect_err)
        .ok()
        .map(to_heap)
}
