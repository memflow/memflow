use super::super::VirtualMemoryInstance;
use super::super::{util::*, COptArc, GenericCloneTable, OpaqueCloneTable};
use super::OptionArchitectureIdent;
use super::{MuAddress, MuModuleInfo};
use crate::architecture::ArchitectureIdent;
use crate::error::*;
use crate::os::{ModuleAddressCallback, ModuleInfo, Process, ProcessInfo};
use crate::types::Address;
use std::ffi::c_void;

use libloading::Library;

pub type OpaqueProcessFunctionTable = ProcessFunctionTable<c_void>;

impl Copy for OpaqueProcessFunctionTable {}

impl Clone for OpaqueProcessFunctionTable {
    fn clone(&self) -> Self {
        *self
    }
}

#[repr(C)]
pub struct ProcessFunctionTable<T> {
    pub module_address_list_callback: extern "C" fn(
        process: &mut T,
        target_arch: OptionArchitectureIdent,
        callback: ModuleAddressCallback,
    ) -> i32,
    pub module_by_address: extern "C" fn(
        process: &mut T,
        address: Address,
        architecture: ArchitectureIdent,
        out: &mut MuModuleInfo,
    ) -> i32,
    pub primary_module_address: extern "C" fn(process: &mut T, out: &mut MuAddress) -> i32,
    pub info: extern "C" fn(process: &T) -> &ProcessInfo,
    pub virt_mem: extern "C" fn(process: &mut T) -> &mut c_void,
    pub drop: unsafe extern "C" fn(thisptr: &mut T),
}

impl<T: Process> Default for ProcessFunctionTable<T> {
    fn default() -> Self {
        Self {
            module_address_list_callback: c_module_address_list_callback,
            module_by_address: c_module_by_address,
            primary_module_address: c_primary_module_address,
            info: c_info,
            virt_mem: c_virt_mem,
            drop: c_drop::<T>,
        }
    }
}

impl<T: Process> ProcessFunctionTable<T> {
    pub fn into_opaque(self) -> OpaqueProcessFunctionTable {
        unsafe { std::mem::transmute(self) }
    }
}

extern "C" fn c_virt_mem<T: Process>(process: &mut T) -> &mut c_void {
    unsafe {
        (process.virt_mem() as *mut _ as *mut c_void)
            .as_mut()
            .unwrap()
    }
}

extern "C" fn c_module_address_list_callback<T: Process>(
    process: &mut T,
    target_arch: OptionArchitectureIdent,
    callback: ModuleAddressCallback,
) -> i32 {
    process
        .module_address_list_callback(target_arch, callback)
        .into_int_result()
}

extern "C" fn c_module_by_address<T: Process>(
    process: &mut T,
    address: Address,
    target_arch: ArchitectureIdent,
    out: &mut MuModuleInfo,
) -> i32 {
    process
        .module_by_address(address, target_arch)
        .into_int_out_result(out)
}

extern "C" fn c_primary_module_address<T: Process>(process: &mut T, out: &mut MuAddress) -> i32 {
    process.primary_module_address().into_int_out_result(out)
}

extern "C" fn c_info<T: Process>(process: &T) -> &ProcessInfo {
    process.info()
}

#[repr(C)]
pub struct PluginProcess<'a> {
    instance: &'a mut c_void,
    vtable: OpaqueProcessFunctionTable,
    virt_mem: VirtualMemoryInstance<'a>,
}

impl<'a> PluginProcess<'a> {
    pub fn new<T: 'a + Process>(proc: T) -> Self {
        let instance = Box::leak(Box::new(proc));
        let instance_void = unsafe { (instance as *mut T as *mut c_void).as_mut() }.unwrap();
        let vtable = ProcessFunctionTable::<T>::default().into_opaque();
        let virt_mem = unsafe {
            VirtualMemoryInstance::unsafe_new::<T::VirtualMemoryType>(c_virt_mem(instance))
        };
        Self {
            instance: instance_void,
            vtable,
            virt_mem,
        }
    }
}

impl<'a> Drop for PluginProcess<'a> {
    fn drop(&mut self) {
        unsafe { (self.vtable.drop)(self.instance) };
    }
}

impl<'a> Process for PluginProcess<'a> {
    type VirtualMemoryType = VirtualMemoryInstance<'a>;

    fn virt_mem(&mut self) -> &mut Self::VirtualMemoryType {
        &mut self.virt_mem
    }

    fn module_address_list_callback(
        &mut self,
        target_arch: OptionArchitectureIdent,
        callback: ModuleAddressCallback,
    ) -> Result<()> {
        let res = (self.vtable.module_address_list_callback)(self.instance, target_arch, callback);
        result_from_int_void(res)
    }

    fn module_by_address(
        &mut self,
        address: Address,
        architecture: ArchitectureIdent,
    ) -> Result<ModuleInfo> {
        let mut out = MuModuleInfo::uninit();
        let res = (self.vtable.module_by_address)(self.instance, address, architecture, &mut out);
        result_from_int(res, out)
    }

    fn primary_module_address(&mut self) -> Result<Address> {
        let mut out = MuAddress::uninit();
        let res = (self.vtable.primary_module_address)(self.instance, &mut out);
        result_from_int(res, out)
    }

    fn info(&self) -> &ProcessInfo {
        (self.vtable.info)(self.instance)
    }
}

#[repr(C)]
pub struct ArcPluginProcess {
    inner: PluginProcess<'static>,
    clone: OpaqueCloneTable,
    library: COptArc<Library>,
}

impl ArcPluginProcess {
    pub fn new<T: 'static + Process + Clone>(proc: T, lib: COptArc<Library>) -> Self {
        Self {
            inner: PluginProcess::new(proc),
            clone: GenericCloneTable::<T>::default().into_opaque(),
            library: lib,
        }
    }
}

impl Clone for ArcPluginProcess {
    fn clone(&self) -> Self {
        let instance = (self.clone.clone)(self.inner.instance).expect("Unable to clone Process");
        let vmem_ref =
            (self.inner.vtable.virt_mem)(unsafe { (instance as *mut c_void).as_mut() }.unwrap());
        Self {
            inner: PluginProcess {
                instance,
                vtable: self.inner.vtable,
                virt_mem: VirtualMemoryInstance {
                    instance: vmem_ref,
                    vtable: self.inner.virt_mem.vtable,
                },
            },
            clone: self.clone,
            library: self.library.clone(),
        }
    }
}

impl Process for ArcPluginProcess {
    type VirtualMemoryType = VirtualMemoryInstance<'static>;

    fn virt_mem(&mut self) -> &mut Self::VirtualMemoryType {
        self.inner.virt_mem()
    }

    fn module_address_list_callback(
        &mut self,
        target_arch: OptionArchitectureIdent,
        callback: ModuleAddressCallback,
    ) -> Result<()> {
        self.inner
            .module_address_list_callback(target_arch, callback)
    }

    fn module_by_address(
        &mut self,
        address: Address,
        architecture: ArchitectureIdent,
    ) -> Result<ModuleInfo> {
        self.inner.module_by_address(address, architecture)
    }

    fn primary_module_address(&mut self) -> Result<Address> {
        self.inner.primary_module_address()
    }

    fn info(&self) -> &ProcessInfo {
        self.inner.info()
    }
}
