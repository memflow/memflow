use super::super::super::{util::*, COptArc, GenericCloneTable, OpaqueCloneTable};
use crate::connector::cpu_state::CpuState;

use std::ffi::c_void;

use libloading::Library;

pub type OpaqueCpuStateFunctionTable = CpuStateFunctionTable<c_void>;

impl Copy for OpaqueCpuStateFunctionTable {}

impl Clone for OpaqueCpuStateFunctionTable {
    fn clone(&self) -> Self {
        *self
    }
}

#[repr(C)]
pub struct CpuStateFunctionTable<T> {
    // TODO: stuff
    pub drop: unsafe extern "C" fn(thisptr: &mut T),
}

impl<T: CpuState> Default for CpuStateFunctionTable<T> {
    fn default() -> Self {
        Self { drop: c_drop::<T> }
    }
}

impl<T: CpuState> CpuStateFunctionTable<T> {
    pub fn into_opaque(self) -> OpaqueCpuStateFunctionTable {
        unsafe { std::mem::transmute(self) }
    }
}

#[repr(C)]
pub struct PluginCpuState<'a> {
    instance: &'a mut c_void,
    vtable: OpaqueCpuStateFunctionTable,
    library: COptArc<Library>,
}

impl<'a> PluginCpuState<'a> {
    pub fn new<T: 'a + CpuState>(cpu_state: T, lib: COptArc<Library>) -> Self {
        let instance = Box::leak(Box::new(cpu_state));
        let instance_void = unsafe { (instance as *mut T as *mut c_void).as_mut() }.unwrap();
        let vtable = CpuStateFunctionTable::<T>::default().into_opaque();
        Self {
            instance: instance_void,
            vtable,
            library: lib,
        }
    }
}

impl<'a> Drop for PluginCpuState<'a> {
    fn drop(&mut self) {
        unsafe { (self.vtable.drop)(self.instance) };
    }
}

impl<'a> CpuState for PluginCpuState<'a> {}

#[repr(C)]
pub struct ArcPluginCpuState {
    inner: PluginCpuState<'static>,
    clone: OpaqueCloneTable,
}

impl ArcPluginCpuState {
    pub fn new<T: 'static + CpuState + Clone>(cpu_state: T, lib: COptArc<Library>) -> Self {
        Self {
            inner: PluginCpuState::new(cpu_state, lib),
            clone: GenericCloneTable::<T>::default().into_opaque(),
        }
    }
}

impl Clone for ArcPluginCpuState {
    fn clone(&self) -> Self {
        let instance = (self.clone.clone)(self.inner.instance).expect("Unable to clone CpuState");
        Self {
            inner: PluginCpuState {
                instance,
                vtable: self.inner.vtable,
                library: self.inner.library.clone(),
            },
            clone: self.clone,
        }
    }
}

impl CpuState for ArcPluginCpuState {}
