//use crate::error::*;
use crate::os::*;
use crate::types::Address;

pub mod kernel;
pub use kernel::{KernelFunctionTable, KernelInstance, OpaqueKernelFunctionTable};

pub mod process;
pub use process::{ArcPluginProcess, PluginProcess};

use super::{
    GenericBaseTable, //MEMFLOW_PLUGIN_VERSION,
    OpaqueBaseTable,
    /*Args, LibInstance, Loadable,*/ OpaquePhysicalMemoryFunctionTable,
    OpaqueVirtualMemoryFunctionTable,
};

//use libloading::Library;
//use std::sync::Arc;

//use log::*;

use std::mem::MaybeUninit;

// Type aliases needed for &mut MaybeUninit<T> to work with bindgen
pub type MUProcessInfo = MaybeUninit<ProcessInfo>;
pub type MUModuleInfo = MaybeUninit<ModuleInfo>;
pub type MUPluginProcess<'a> = MaybeUninit<PluginProcess<'a>>;
pub type MUAddress = MaybeUninit<Address>;

pub type OptionArchitectureIdent<'a> = Option<&'a crate::architecture::ArchitectureIdent>;

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
    pub base: OpaqueBaseTable,
    /// The vtable for all kernel functions
    pub kernel: OpaqueKernelFunctionTable,
    /// The vtable for all physical memory access if available
    pub phys: Option<&'static OpaquePhysicalMemoryFunctionTable>,
    /// The vtable for all virtual memory access if available
    pub virt: Option<&'static OpaqueVirtualMemoryFunctionTable>,
}

impl OSLayerFunctionTable {
    pub fn new<'a, T: 'static + Kernel<'a> + Clone>() -> Self {
        OSLayerFunctionTable {
            base: GenericBaseTable::<T>::new().into_opaque(),
            kernel: KernelFunctionTable::<T>::default().into_opaque(),
            phys: None,
            virt: None,
        }
    }
}
