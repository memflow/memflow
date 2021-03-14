use crate::error::*;

use super::super::super::{util::*, COptArc, GenericCloneTable, OpaqueCloneTable};
use super::ArcPluginKeyboardState;
use crate::os::Keyboard;

use std::ffi::c_void;

use super::super::MUArcPluginKeyboardState;

use libloading::Library;

pub type OpaqueKeyboardFunctionTable = KeyboardFunctionTable<c_void>;

impl Copy for OpaqueKeyboardFunctionTable {}

impl Clone for OpaqueKeyboardFunctionTable {
    fn clone(&self) -> Self {
        *self
    }
}

#[repr(C)]
pub struct KeyboardFunctionTable<T> {
    pub state: extern "C" fn(
        keyboard: &mut T,
        lib: COptArc<Library>,
        out: &mut MUArcPluginKeyboardState,
    ) -> i32,
    pub set_state: extern "C" fn(keyboard: &mut T, state: &ArcPluginKeyboardState) -> i32,
    pub drop: unsafe extern "C" fn(thisptr: &mut T),
}

impl<T: Keyboard> Default for KeyboardFunctionTable<T> {
    fn default() -> Self {
        Self {
            state: c_state,
            set_state: c_set_state,
            drop: c_drop::<T>,
        }
    }
}

impl<T: Keyboard> KeyboardFunctionTable<T> {
    pub fn into_opaque(self) -> OpaqueKeyboardFunctionTable {
        unsafe { std::mem::transmute(self) }
    }
}

extern "C" fn c_state<T: Keyboard>(
    keyboard: &mut T,
    lib: COptArc<Library>,
    out: &mut MUArcPluginKeyboardState,
) -> i32 {
    keyboard
        .state()
        .map(|ks| ArcPluginKeyboardState::new(ks, lib))
        .into_int_out_result(out)
}

/// # Safety
/// This is just inherently unsafe. Only pass a state you got from the state() function into this.
extern "C" fn c_set_state<T: Keyboard>(keyboard: &mut T, state: &ArcPluginKeyboardState) -> i32 {
    keyboard
        .set_state(unsafe { &*(state.instance as *const c_void as *const T::KeyboardStateType) })
        .into_int_result()
}

#[repr(C)]
pub struct PluginKeyboard<'a> {
    instance: &'a mut c_void,
    vtable: OpaqueKeyboardFunctionTable,
    library: COptArc<Library>,
}

impl<'a> PluginKeyboard<'a> {
    pub fn new<T: 'a + Keyboard>(keyboard: T, lib: COptArc<Library>) -> Self {
        let instance = Box::leak(Box::new(keyboard));
        let instance_void = unsafe { (instance as *mut T as *mut c_void).as_mut() }.unwrap();
        let vtable = KeyboardFunctionTable::<T>::default().into_opaque();
        Self {
            instance: instance_void,
            vtable,
            library: lib,
        }
    }
}

impl<'a> Drop for PluginKeyboard<'a> {
    fn drop(&mut self) {
        unsafe { (self.vtable.drop)(self.instance) };
    }
}

impl<'a> Keyboard for PluginKeyboard<'a> {
    type KeyboardStateType = ArcPluginKeyboardState;

    fn state(&mut self) -> Result<Self::KeyboardStateType> {
        let mut out = MUArcPluginKeyboardState::uninit();
        let res = (self.vtable.state)(self.instance, self.library.clone(), &mut out);
        result_from_int(res, out)
    }

    fn set_state(&mut self, state: &Self::KeyboardStateType) -> Result<()> {
        let res = (self.vtable.set_state)(self.instance, state);
        result_from_int_void(res)
    }
}

#[repr(C)]
pub struct ArcPluginKeyboard {
    inner: PluginKeyboard<'static>,
    clone: OpaqueCloneTable,
}

impl ArcPluginKeyboard {
    pub fn new<T: 'static + Keyboard + Clone>(keyboard: T, lib: COptArc<Library>) -> Self {
        Self {
            inner: PluginKeyboard::new(keyboard, lib),
            clone: GenericCloneTable::<T>::default().into_opaque(),
        }
    }
}

impl Clone for ArcPluginKeyboard {
    fn clone(&self) -> Self {
        let instance = (self.clone.clone)(self.inner.instance).expect("Unable to clone Keyboard");
        Self {
            inner: PluginKeyboard {
                instance,
                vtable: self.inner.vtable,
                library: self.inner.library.clone(),
            },
            clone: self.clone,
        }
    }
}

impl Keyboard for ArcPluginKeyboard {
    type KeyboardStateType = ArcPluginKeyboardState;

    fn state(&mut self) -> Result<Self::KeyboardStateType> {
        self.inner.state()
    }

    fn set_state(&mut self, state: &Self::KeyboardStateType) -> Result<()> {
        self.inner.set_state(state)
    }
}
