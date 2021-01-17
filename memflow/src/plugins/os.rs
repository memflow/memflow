//use crate::error::*;
use crate::mem::{/*VirtualMemory,*/ VirtualReadData, VirtualWriteData};
use crate::os::*;
use crate::types::Address;

use super::util::*;
use super::{
    /*Args, LibInstance, Loadable,*/ OpaquePhysicalMemoryFunctionTable,
    OptionVoid,
    //MEMFLOW_PLUGIN_VERSION,
};

use std::ffi::c_void; //, CString};
use std::os::raw::c_char;
//use std::path::Path;
use std::sync::Arc;

use libloading::Library;

//use log::*;

#[repr(C)]
pub struct OSLayerDescriptor {
    /// The connector inventory api version for when the connector was built.
    /// This has to be set to `MEMFLOW_PLUGIN_VERSION` of memflow.
    ///
    /// If the versions mismatch the inventory will refuse to load.
    pub connector_version: i32,

    /// The name of the connector.
    /// This name will be used when loading a connector from a connector inventory.
    pub name: &'static str,

    /// The vtable for all opaque function calls to the connector.
    pub create_vtable: extern "C" fn() -> OSLayerFunctionTable,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct OSLayerFunctionTable {
    /// The vtable for object creation and cloning
    pub base: KernelBaseTable,
    /// The vtable for all kernel functions
    pub kernel: OpaqueKernelFunctionTable,
    /// The vtable for all physical memory access if available
    pub phys: Option<&'static OpaquePhysicalMemoryFunctionTable>,
    /// The vtable for all virtual memory access if available
    pub virt: Option<&'static VirtualMemoryFunctionTable>,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct KernelBaseTable {
    pub create: extern "C" fn(args: *const c_char, log_level: i32) -> OptionVoid,
    pub clone: extern "C" fn(kernel: &c_void) -> OptionVoid,
    pub drop: extern "C" fn(kernel: &mut c_void),
}

impl KernelBaseTable {
    pub fn new<'a, T: Kernel<'a> + Clone>(
        create: extern "C" fn(*const c_char, i32) -> OptionVoid,
    ) -> Self {
        Self {
            create,
            clone: c_clone::<T>,
            drop: c_drop::<T>,
        }
    }
}

pub type PluginAddressCallback<'a> = AddressCallback<'a, c_void>;

pub type OpaqueKernelFunctionTable = KernelFunctionTable<'static, c_void>;

impl Copy for OpaqueKernelFunctionTable {}

impl Clone for OpaqueKernelFunctionTable {
    fn clone(&self) -> Self {
        *self
    }
}

#[repr(C)]
pub struct KernelFunctionTable<'a, T> {
    pub process_address_list_callback:
        extern "C" fn(kernel: &mut T, callback: AddressCallback<T>) -> i32,
    pub process_info_by_address:
        extern "C" fn(kernel: &mut T, address: Address, out: &mut ProcessInfo) -> i32,
    pub process_by_info:
        extern "C" fn(kernel: &'a mut T, info: ProcessInfo, out: &mut PluginProcessRef) -> i32,
    pub module_address_list_callback:
        extern "C" fn(kernel: &mut T, callback: AddressCallback<T>) -> i32,
    pub module_by_address:
        extern "C" fn(kernel: &mut T, address: Address, out: &mut ModuleInfo) -> i32,
    pub info: extern "C" fn(kernel: &T) -> &KernelInfo,
}

impl<'a, T: Kernel<'a>> Default for KernelFunctionTable<'a, T> {
    fn default() -> Self {
        Self {
            process_address_list_callback: c_process_address_list_callback,
            process_info_by_address: c_process_info_by_address,
            process_by_info: c_process_by_info,
            module_address_list_callback: c_module_address_list_callback,
            module_by_address: c_module_by_address,
            info: c_kernel_info,
        }
    }
}

impl<'a, T: Kernel<'a>> KernelFunctionTable<'a, T> {
    pub fn into_opaque(self) -> OpaqueKernelFunctionTable {
        unsafe { std::mem::transmute(self) }
    }
}

extern "C" fn c_process_address_list_callback<'a, T: Kernel<'a>>(
    kernel: &mut T,
    callback: AddressCallback<T>,
) -> i32 {
    kernel.process_address_list_callback(callback).int_result()
}

extern "C" fn c_process_info_by_address<'a, T: Kernel<'a>>(
    kernel: &mut T,
    address: Address,
    out: &mut ProcessInfo,
) -> i32 {
    kernel.process_info_by_address(address).int_out_result(out)
}

extern "C" fn c_process_by_info<'a, T: 'a + Kernel<'a>>(
    kernel: &'a mut T,
    info: ProcessInfo,
    out: &mut PluginProcessRef,
) -> i32 {
    kernel
        .process_by_info(info)
        .map(|p| unsafe { PluginProcessRef::from(p) })
        .int_out_result(out)
}

extern "C" fn c_module_address_list_callback<'a, T: Kernel<'a>>(
    kernel: &mut T,
    callback: AddressCallback<T>,
) -> i32 {
    kernel.module_address_list_callback(callback).int_result()
}

extern "C" fn c_module_by_address<'a, T: Kernel<'a>>(
    kernel: &mut T,
    address: Address,
    out: &mut ModuleInfo,
) -> i32 {
    kernel.module_by_address(address).int_out_result(out)
}

extern "C" fn c_kernel_info<'a, T: Kernel<'a>>(kernel: &T) -> &KernelInfo {
    kernel.info()
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct VirtualMemoryFunctionTable {
    pub virt_read_raw_list: extern "C" fn(
        virt_mem: &mut c_void,
        read_data: *mut VirtualReadData,
        read_data_count: usize,
    ) -> i32,
    pub virt_write_raw_list: extern "C" fn(
        virt_mem: &mut c_void,
        write_data: *const VirtualWriteData,
        write_data_count: usize,
    ) -> i32,
}
/// Describes initialized kernel instance
///
/// This structure is returned by `Kernel`. It is needed to maintain reference
/// counts to the loaded connector library.
#[repr(C)]
pub struct KernelInstance {
    instance: &'static mut c_void,
    vtable: OSLayerFunctionTable,

    /// Internal library arc.
    ///
    /// This will keep the library loaded in memory as long as the kernel instance is alive.
    /// This has to be the last member of the struct so the library will be unloaded _after_
    /// the instance is destroyed.
    ///
    /// If the library is unloaded prior to the instance this will lead to a SIGSEGV.
    library: Arc<Library>,
}

#[repr(C)]
pub struct PluginProcessRef {
    instance: &'static mut c_void,
}

impl PluginProcessRef {
    unsafe fn from<T: Process>(proc: T) -> Self {
        Self {
            instance: to_static_heap(proc),
        }
    }
}
