use crate::error::*;

use super::{
    ArcPluginKeyboard, ArcPluginProcess, Keyboard, MUArcPluginKeyboard, MUPluginKeyboard,
    OSKeyboardFunctionTable, OSLayerFunctionTable, PluginKeyboard, PluginOSKeyboard, PluginProcess,
};
use crate::os::{
    AddressCallback, ModuleInfo, OSInfo, OSInner, OSKeyboardInner, Process, ProcessInfo,
};
use crate::types::Address;
use std::ffi::c_void;

use super::super::COptArc;
use super::PluginOS;
use super::{MUArcPluginProcess, MUModuleInfo, MUPluginProcess, MUProcessInfo};

use libloading::Library;

pub type OpaqueOSFunctionTable = OSFunctionTable<'static, c_void, c_void>;

impl Copy for OpaqueOSFunctionTable {}

impl Clone for OpaqueOSFunctionTable {
    fn clone(&self) -> Self {
        *self
    }
}

#[repr(C)]
pub struct OSFunctionTable<'a, P, T> {
    pub process_address_list_callback: extern "C" fn(os: &mut T, callback: AddressCallback) -> i32,
    pub process_info_by_address:
        extern "C" fn(os: &mut T, address: Address, out: &mut MUProcessInfo) -> i32,
    pub process_by_info:
        extern "C" fn(os: &'a mut T, info: ProcessInfo, out: &mut MUPluginProcess<'a>) -> i32,
    pub into_process_by_info: extern "C" fn(
        os: &mut T,
        info: ProcessInfo,
        lib: COptArc<Library>,
        out: &mut MUArcPluginProcess,
    ) -> i32,
    pub module_address_list_callback: extern "C" fn(os: &mut T, callback: AddressCallback) -> i32,
    pub module_by_address:
        extern "C" fn(os: &mut T, address: Address, out: &mut MUModuleInfo) -> i32,
    pub info: extern "C" fn(os: &T) -> &OSInfo,
    phantom: std::marker::PhantomData<P>,
}

impl<'a, P: 'static + Process + Clone, T: PluginOS<P>> Default for &'a OSFunctionTable<'a, P, T> {
    fn default() -> Self {
        &OSFunctionTable {
            process_address_list_callback: c_process_address_list_callback,
            process_info_by_address: c_process_info_by_address,
            process_by_info: c_process_by_info,
            into_process_by_info: c_into_process_by_info,
            module_address_list_callback: c_module_address_list_callback,
            module_by_address: c_module_by_address,
            info: c_os_info,
            phantom: std::marker::PhantomData {},
        }
    }
}

impl<'a, P: Process + Clone, T: PluginOS<P>> OSFunctionTable<'a, P, T> {
    pub fn as_opaque(&self) -> &OpaqueOSFunctionTable {
        unsafe { &*(self as *const Self as *const OpaqueOSFunctionTable) }
    }
}

extern "C" fn c_process_address_list_callback<'a, T: OSInner<'a>>(
    os: &mut T,
    callback: AddressCallback,
) -> i32 {
    os.process_address_list_callback(callback).as_int_result()
}

extern "C" fn c_process_info_by_address<'a, T: OSInner<'a>>(
    os: &mut T,
    address: Address,
    out: &mut MUProcessInfo,
) -> i32 {
    os.process_info_by_address(address).as_int_out_result(out)
}

extern "C" fn c_process_by_info<'a, T: 'a + OSInner<'a>>(
    os: &'a mut T,
    info: ProcessInfo,
    out: &mut MUPluginProcess<'a>,
) -> i32 {
    os.process_by_info(info)
        .map(PluginProcess::new)
        .as_int_out_result(out)
}

extern "C" fn c_into_process_by_info<P: 'static + Process + Clone, T: 'static + PluginOS<P>>(
    os: &mut T,
    info: ProcessInfo,
    lib: COptArc<Library>,
    out: &mut MUArcPluginProcess,
) -> i32 {
    let os = unsafe { Box::from_raw(os) };
    os.into_process_by_info(info)
        .map(|p| ArcPluginProcess::new(p, lib))
        .as_int_out_result(out)
}

extern "C" fn c_module_address_list_callback<'a, T: OSInner<'a>>(
    os: &mut T,
    callback: AddressCallback,
) -> i32 {
    os.module_address_list_callback(callback).as_int_result()
}

extern "C" fn c_module_by_address<'a, T: OSInner<'a>>(
    os: &mut T,
    address: Address,
    out: &mut MUModuleInfo,
) -> i32 {
    os.module_by_address(address).as_int_out_result(out)
}

extern "C" fn c_os_info<'a, T: OSInner<'a>>(os: &T) -> &OSInfo {
    os.info()
}

/// Describes initialized os instance
///
/// This structure is returned by `OS`. It is needed to maintain reference
/// counts to the loaded plugin library.
#[repr(C)]
pub struct OSInstance {
    instance: &'static mut c_void,
    vtable: OSLayerFunctionTable,

    /// Internal library arc.
    ///
    /// This will keep the library loaded in memory as long as the os instance is alive.
    /// This has to be the last member of the struct so the library will be unloaded _after_
    /// the instance is destroyed.
    ///
    /// If the library is unloaded prior to the instance this will lead to a SIGSEGV.
    pub(super) library: COptArc<Library>,
}

impl OSInstance {
    pub fn builder<P: 'static + Process + Clone, T: PluginOS<P>>(
        instance: T,
    ) -> OSInstanceBuilder<T> {
        OSInstanceBuilder {
            instance,
            vtable: OSLayerFunctionTable::new::<P, T>(),
        }
    }
}

/// Builder for the os instance structure.
pub struct OSInstanceBuilder<T> {
    instance: T,
    vtable: OSLayerFunctionTable,
}

impl<T> OSInstanceBuilder<T> {
    /// Enables the optional Keyboard feature for the OSInstance.
    pub fn enable_keyboard<K>(mut self) -> Self
    where
        K: 'static + Keyboard + Clone,
        T: PluginOSKeyboard<K>,
    {
        self.vtable.keyboard = Some(<&OSKeyboardFunctionTable<K, T>>::default().as_opaque());
        self
    }

    /// Build the OSInstance
    pub fn build(self) -> OSInstance {
        OSInstance {
            instance: unsafe {
                Box::into_raw(Box::new(self.instance))
                    .cast::<c_void>()
                    .as_mut()
            }
            .unwrap(),
            vtable: self.vtable,
            library: None.into(),
        }
    }
}

impl OSInstance {
    pub fn has_phys_mem(&self) -> bool {
        self.vtable.phys.is_some()
    }

    pub fn has_virt_mem(&self) -> bool {
        self.vtable.virt.is_some()
    }

    pub fn has_keyboard(&self) -> bool {
        self.vtable.keyboard.is_some()
    }
}

impl<'a> OSInner<'a> for OSInstance {
    type ProcessType = PluginProcess<'a>;
    type IntoProcessType = ArcPluginProcess;

    /// Walks a process list and calls a callback for each process structure address
    ///
    /// The callback is fully opaque. We need this style so that C FFI can work seamlessly.
    fn process_address_list_callback(&mut self, callback: AddressCallback) -> Result<()> {
        result_from_int_void((self.vtable.os.process_address_list_callback)(
            self.instance,
            callback,
        ))
    }

    /// Find process information by its internal address
    fn process_info_by_address(&mut self, address: Address) -> Result<ProcessInfo> {
        let mut out = MUProcessInfo::uninit();
        let res = (self.vtable.os.process_info_by_address)(self.instance, address, &mut out);
        result_from_int(res, out)
    }

    /// Construct a process by its info, borrowing the os
    ///
    /// It will share the underlying memory resources
    fn process_by_info(&'a mut self, info: ProcessInfo) -> Result<Self::ProcessType> {
        let mut out = MUPluginProcess::uninit();
        // Shorten the lifetime of instance
        let instance = unsafe { (self.instance as *mut c_void).as_mut() }.unwrap();
        let res = (self.vtable.os.process_by_info)(instance, info, &mut out);
        result_from_int(res, out)
    }
    /// Construct a process by its info, consuming the os
    ///
    /// This function will consume the OS instance and move its resources into the process
    fn into_process_by_info(mut self, info: ProcessInfo) -> Result<Self::IntoProcessType> {
        let mut out = MUArcPluginProcess::uninit();
        let res = (self.vtable.os.into_process_by_info)(
            self.instance,
            info,
            self.library.take(),
            &mut out,
        );
        std::mem::forget(self);
        result_from_int(res, out)
    }

    /// Walks the os module list and calls the provided callback for each module structure
    /// address
    ///
    /// # Arguments
    /// * `callback` - where to pass each matching module to. This is an opaque callback.
    fn module_address_list_callback(&mut self, callback: AddressCallback) -> Result<()> {
        result_from_int_void((self.vtable.os.module_address_list_callback)(
            self.instance,
            callback,
        ))
    }

    /// Retrieves a module by its structure address
    ///
    /// # Arguments
    /// * `address` - address where module's information resides in
    fn module_by_address(&mut self, address: Address) -> Result<ModuleInfo> {
        let mut out = MUModuleInfo::uninit();
        let res = (self.vtable.os.module_by_address)(self.instance, address, &mut out);
        result_from_int(res, out)
    }

    /// Retrieves the os info
    fn info(&self) -> &OSInfo {
        (self.vtable.os.info)(self.instance)
    }
}

/// Optional Keyboard feature implementation
impl<'a> OSKeyboardInner<'a> for OSInstance {
    type KeyboardType = PluginKeyboard<'a>;
    type IntoKeyboardType = ArcPluginKeyboard;

    fn keyboard(&'a mut self) -> Result<Self::KeyboardType> {
        let kbd = self
            .vtable
            .keyboard
            .ok_or(Error::Connector("unsupported optional feature"))?;
        let mut out = MUPluginKeyboard::uninit();
        // Shorten the lifetime of instance
        let instance = unsafe { (self.instance as *mut c_void).as_mut() }.unwrap();
        let res = (kbd.keyboard)(instance, self.library.clone(), &mut out);
        result_from_int(res, out)
    }

    fn into_keyboard(mut self) -> Result<Self::IntoKeyboardType> {
        let kbd = self
            .vtable
            .keyboard
            .ok_or(Error::Connector("unsupported optional feature"))?;
        let mut out = MUArcPluginKeyboard::uninit();
        let res = (kbd.into_keyboard)(self.instance, self.library.take(), &mut out);
        std::mem::forget(self);
        result_from_int(res, out)
    }
}

impl Clone for OSInstance {
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

impl Drop for OSInstance {
    fn drop(&mut self) {
        unsafe {
            (self.vtable.base.drop)(self.instance);
        }
    }
}
