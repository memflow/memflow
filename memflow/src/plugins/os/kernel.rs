use crate::error::*;

use super::{ArcPluginProcess, OSLayerFunctionTable, PluginProcess};
use crate::os::{AddressCallback, KernelInfo, KernelInner, ModuleInfo, Process, ProcessInfo};
use crate::types::Address;
use std::ffi::c_void;

use super::super::COptArc;
use super::PluginKernel;
use super::{MUArcPluginProcess, MUModuleInfo, MUPluginProcess, MUProcessInfo};

use libloading::Library;

pub type OpaqueKernelFunctionTable = KernelFunctionTable<'static, c_void, c_void>;

impl Copy for OpaqueKernelFunctionTable {}

impl Clone for OpaqueKernelFunctionTable {
    fn clone(&self) -> Self {
        *self
    }
}

#[repr(C)]
pub struct KernelFunctionTable<'a, P, T> {
    pub process_address_list_callback:
        extern "C" fn(kernel: &mut T, callback: AddressCallback) -> i32,
    pub process_info_by_address:
        extern "C" fn(kernel: &mut T, address: Address, out: &mut MUProcessInfo) -> i32,
    pub process_by_info:
        extern "C" fn(kernel: &'a mut T, info: ProcessInfo, out: &mut MUPluginProcess<'a>) -> i32,
    pub into_process_by_info: extern "C" fn(
        kernel: &mut T,
        info: ProcessInfo,
        lib: COptArc<Library>,
        out: &mut MUArcPluginProcess,
    ) -> i32,
    pub module_address_list_callback:
        extern "C" fn(kernel: &mut T, callback: AddressCallback) -> i32,
    pub module_by_address:
        extern "C" fn(kernel: &mut T, address: Address, out: &mut MUModuleInfo) -> i32,
    pub info: extern "C" fn(kernel: &T) -> &KernelInfo,
    phantom: std::marker::PhantomData<P>,
}

impl<'a, P: 'static + Process + Clone, T: PluginKernel<P>> Default
    for KernelFunctionTable<'a, P, T>
{
    fn default() -> Self {
        Self {
            process_address_list_callback: c_process_address_list_callback,
            process_info_by_address: c_process_info_by_address,
            process_by_info: c_process_by_info,
            into_process_by_info: c_into_process_by_info,
            module_address_list_callback: c_module_address_list_callback,
            module_by_address: c_module_by_address,
            info: c_kernel_info,
            phantom: Default::default(),
        }
    }
}

impl<'a, P: Process + Clone, T: PluginKernel<P>> KernelFunctionTable<'a, P, T> {
    pub fn into_opaque(self) -> OpaqueKernelFunctionTable {
        unsafe { std::mem::transmute(self) }
    }
}

extern "C" fn c_process_address_list_callback<'a, T: KernelInner<'a>>(
    kernel: &mut T,
    callback: AddressCallback,
) -> i32 {
    kernel.process_address_list_callback(callback).int_result()
}

extern "C" fn c_process_info_by_address<'a, T: KernelInner<'a>>(
    kernel: &mut T,
    address: Address,
    out: &mut MUProcessInfo,
) -> i32 {
    kernel.process_info_by_address(address).int_out_result(out)
}

extern "C" fn c_process_by_info<'a, T: 'a + KernelInner<'a>>(
    kernel: &'a mut T,
    info: ProcessInfo,
    out: &mut MUPluginProcess<'a>,
) -> i32 {
    kernel
        .process_by_info(info)
        .map(PluginProcess::new)
        .int_out_result(out)
}

extern "C" fn c_into_process_by_info<P: 'static + Process + Clone, T: 'static + PluginKernel<P>>(
    kernel: &mut T,
    info: ProcessInfo,
    lib: COptArc<Library>,
    out: &mut MUArcPluginProcess,
) -> i32 {
    let kernel = unsafe { Box::from_raw(kernel) };
    kernel
        .into_process_by_info(info)
        .map(|p| ArcPluginProcess::new(p, lib))
        .int_out_result(out)
}

extern "C" fn c_module_address_list_callback<'a, T: KernelInner<'a>>(
    kernel: &mut T,
    callback: AddressCallback,
) -> i32 {
    kernel.module_address_list_callback(callback).int_result()
}

extern "C" fn c_module_by_address<'a, T: KernelInner<'a>>(
    kernel: &mut T,
    address: Address,
    out: &mut MUModuleInfo,
) -> i32 {
    kernel.module_by_address(address).int_out_result(out)
}

extern "C" fn c_kernel_info<'a, T: KernelInner<'a>>(kernel: &T) -> &KernelInfo {
    kernel.info()
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
    pub(super) library: COptArc<Library>,
}

impl KernelInstance {
    pub fn new<P: 'static + Process + Clone, T: PluginKernel<P>>(instance: T) -> Self {
        Self {
            instance: unsafe { Box::into_raw(Box::new(instance)).cast::<c_void>().as_mut() }
                .unwrap(),
            vtable: OSLayerFunctionTable::new::<P, T>(),
            library: None.into(),
        }
    }
}

impl<'a> KernelInner<'a> for KernelInstance {
    type ProcessType = PluginProcess<'a>;
    type IntoProcessType = ArcPluginProcess;

    /// Walks a process list and calls a callback for each process structure address
    ///
    /// The callback is fully opaque. We need this style so that C FFI can work seamlessly.
    fn process_address_list_callback(&mut self, callback: AddressCallback) -> Result<()> {
        result_from_int_void((self.vtable.kernel.process_address_list_callback)(
            self.instance,
            callback,
        ))
    }

    /// Find process information by its internal address
    fn process_info_by_address(&mut self, address: Address) -> Result<ProcessInfo> {
        let mut out = MUProcessInfo::uninit();
        let res = (self.vtable.kernel.process_info_by_address)(self.instance, address, &mut out);
        result_from_int(res, out)
    }

    /// Construct a process by its info, borrowing the kernel
    ///
    /// It will share the underlying memory resources
    fn process_by_info(&'a mut self, info: ProcessInfo) -> Result<Self::ProcessType> {
        let mut out = MUPluginProcess::uninit();
        // Shorten the lifetime of instance
        let instance = unsafe { (self.instance as *mut c_void).as_mut() }.unwrap();
        let res = (self.vtable.kernel.process_by_info)(instance, info, &mut out);
        result_from_int(res, out)
    }
    /// Construct a process by its info, consuming the kernel
    ///
    /// This function will consume the Kernel instance and move its resources into the process
    fn into_process_by_info(mut self, info: ProcessInfo) -> Result<Self::IntoProcessType> {
        let mut out = MUArcPluginProcess::uninit();
        let res = (self.vtable.kernel.into_process_by_info)(
            self.instance,
            info,
            self.library.take(),
            &mut out,
        );
        std::mem::forget(self);
        result_from_int(res, out)
    }

    /// Walks the kernel module list and calls the provided callback for each module structure
    /// address
    ///
    /// # Arguments
    /// * `callback` - where to pass each matching module to. This is an opaque callback.
    fn module_address_list_callback(&mut self, callback: AddressCallback) -> Result<()> {
        result_from_int_void((self.vtable.kernel.module_address_list_callback)(
            self.instance,
            callback,
        ))
    }

    /// Retreives a module by its structure address
    ///
    /// # Arguments
    /// * `address` - address where module's information resides in
    fn module_by_address(&mut self, address: Address) -> Result<ModuleInfo> {
        let mut out = MUModuleInfo::uninit();
        let res = (self.vtable.kernel.module_by_address)(self.instance, address, &mut out);
        result_from_int(res, out)
    }

    /// Retreives the kernel info
    fn info(&self) -> &KernelInfo {
        (self.vtable.kernel.info)(self.instance)
    }
}

impl Clone for KernelInstance {
    fn clone(&self) -> Self {
        let instance =
            (self.vtable.base.clone.clone)(self.instance).expect("Unable to clone Connector");
        Self {
            instance,
            vtable: self.vtable,
            library: self.library.clone(),
        }
    }
}

impl Drop for KernelInstance {
    fn drop(&mut self) {
        unsafe {
            (self.vtable.base.drop)(self.instance);
        }
    }
}
