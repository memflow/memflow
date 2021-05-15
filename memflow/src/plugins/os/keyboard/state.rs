use super::super::super::{util::*, GenericCloneTable, OpaqueCloneTable};
use crate::os::KeyboardState;
use crate::types::cglue::COptArc;
use std::ffi::c_void;

use libloading::Library;

pub type OpaqueKeyboardStateFunctionTable = KeyboardStateFunctionTable<c_void>;

impl Copy for OpaqueKeyboardStateFunctionTable {}

impl Clone for OpaqueKeyboardStateFunctionTable {
    fn clone(&self) -> Self {
        *self
    }
}

#[repr(C)]
pub struct KeyboardStateFunctionTable<T> {
    pub is_down: extern "C" fn(keyboard_state: &T, vk: i32) -> i32,
    pub set_down: extern "C" fn(keyboard_state: &mut T, vk: i32, down: i32),
    pub drop: unsafe extern "C" fn(thisptr: &mut T),
}

impl<T: KeyboardState> Default for KeyboardStateFunctionTable<T> {
    fn default() -> Self {
        Self {
            is_down: c_is_down,
            set_down: c_set_down,
            drop: c_drop::<T>,
        }
    }
}

impl<T: KeyboardState> KeyboardStateFunctionTable<T> {
    pub fn into_opaque(self) -> OpaqueKeyboardStateFunctionTable {
        unsafe { std::mem::transmute(self) }
    }
}

extern "C" fn c_is_down<T: KeyboardState>(keyboard_state: &T, vk: i32) -> i32 {
    match keyboard_state.is_down(vk) {
        true => 1,
        false => 0,
    }
}

extern "C" fn c_set_down<T: KeyboardState>(keyboard_state: &mut T, vk: i32, down: i32) {
    if down != 0 {
        keyboard_state.set_down(vk, true);
    } else {
        keyboard_state.set_down(vk, false);
    }
}

#[repr(C)]
pub struct ArcPluginKeyboardState {
    pub(crate) instance: &'static mut c_void,
    vtable: OpaqueKeyboardStateFunctionTable,
    clone: OpaqueCloneTable,
    library: COptArc<Library>,
}

impl ArcPluginKeyboardState {
    pub fn new<T: 'static + KeyboardState + Clone>(
        keyboard_state: T,
        lib: COptArc<Library>,
    ) -> Self {
        let instance = Box::leak(Box::new(keyboard_state));
        let instance_void = unsafe { (instance as *mut T as *mut c_void).as_mut() }.unwrap();
        let vtable = KeyboardStateFunctionTable::<T>::default().into_opaque();
        Self {
            instance: instance_void,
            vtable,
            clone: GenericCloneTable::<T>::default().into_opaque(),
            library: lib,
        }
    }
}

impl Drop for ArcPluginKeyboardState {
    fn drop(&mut self) {
        unsafe { (self.vtable.drop)(self.instance) };
    }
}

impl Clone for ArcPluginKeyboardState {
    fn clone(&self) -> Self {
        let instance = (self.clone.clone)(self.instance).expect("Unable to clone KeyboardState");
        Self {
            instance,
            vtable: self.vtable,
            clone: self.clone,
            library: self.library.clone(),
        }
    }
}

impl KeyboardState for ArcPluginKeyboardState {
    fn is_down(&self, vk: i32) -> bool {
        (self.vtable.is_down)(self.instance, vk) != 0
    }

    fn set_down(&mut self, vk: i32, down: bool) {
        if down {
            (self.vtable.set_down)(self.instance, vk, 1);
        } else {
            (self.vtable.set_down)(self.instance, vk, 0);
        }
    }
}
