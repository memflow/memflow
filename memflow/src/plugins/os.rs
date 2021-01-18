//use crate::error::*;
use crate::os::*;

pub mod kernel;
pub use kernel::{KernelFunctionTable, KernelInstance, OpaqueKernelFunctionTable};

pub mod process;
pub use process::{ArcPluginProcess, PluginProcess};

use super::{
    GenericBaseTable, //MEMFLOW_PLUGIN_VERSION,
    OpaqueBaseTable,
    /*Args, LibInstance, Loadable,*/ OpaquePhysicalMemoryFunctionTable,
    OpaqueVirtualMemoryFunctionTable,
    OptionMut,
};

use std::ffi::c_void; //, CString};
                      //use std::path::Path;
use std::mem::MaybeUninit;

use libloading::Library;
use std::sync::Arc;

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

pub type OSBaseTable = OpaqueBaseTable<OptionMut<c_void>>;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct OSLayerFunctionTable {
    /// The vtable for object creation and cloning
    pub base: OSBaseTable,
    /// The vtable for all kernel functions
    pub kernel: OpaqueKernelFunctionTable,
    /// The vtable for all physical memory access if available
    pub phys: Option<&'static OpaquePhysicalMemoryFunctionTable>,
    /// The vtable for all virtual memory access if available
    pub virt: Option<&'static OpaqueVirtualMemoryFunctionTable>,
}

extern "C" fn none_create<T, I>(_: *const i8, _: Option<&mut I>, _: i32) -> Option<&'static mut T> {
    None
}

impl OSLayerFunctionTable {
    pub fn new<'a, T: 'static + Kernel<'a> + Clone>() -> Self {
        OSLayerFunctionTable {
            base: GenericBaseTable::<T, Option<&'static mut c_void>>::new(none_create)
                .into_opaque(),
            kernel: KernelFunctionTable::<T>::default().into_opaque(),
            phys: None,
            virt: None,
        }
    }
}

pub type PluginAddressCallback<'a> = AddressCallback<'a, c_void>;
