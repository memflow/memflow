pub mod plugin;
pub use plugin::{ArcPluginKeyboard, PluginKeyboard};

pub mod state;
pub use state::ArcPluginKeyboardState;

use crate::error::*;

use crate::os::{Keyboard, OsKeyboardInner};
use std::ffi::c_void;

use super::super::COptArc;
use super::PluginOsKeyboard;
use super::{MuArcPluginKeyboard, MuPluginKeyboard};

use libloading::Library;

pub type OpaqueOsKeyboardFunctionTable = OsKeyboardFunctionTable<'static, c_void, c_void>;

impl Copy for OpaqueOsKeyboardFunctionTable {}

impl Clone for OpaqueOsKeyboardFunctionTable {
    fn clone(&self) -> Self {
        *self
    }
}

#[repr(C)]
pub struct OsKeyboardFunctionTable<'a, K, T> {
    pub keyboard:
        extern "C" fn(os: &'a mut T, lib: COptArc<Library>, out: &mut MuPluginKeyboard<'a>) -> i32,
    pub into_keyboard:
        extern "C" fn(os: &mut T, lib: COptArc<Library>, out: &mut MuArcPluginKeyboard) -> i32,
    phantom: std::marker::PhantomData<K>,
}

impl<'a, K: 'static + Keyboard + Clone, T: PluginOsKeyboard<K>> Default
    for &'a OsKeyboardFunctionTable<'a, K, T>
{
    fn default() -> Self {
        &OsKeyboardFunctionTable {
            keyboard: c_keyboard,
            into_keyboard: c_into_keyboard,
            phantom: std::marker::PhantomData {},
        }
    }
}

impl<'a, P: 'static + Keyboard + Clone, T: PluginOsKeyboard<P>> OsKeyboardFunctionTable<'a, P, T> {
    pub fn as_opaque(&self) -> &OpaqueOsKeyboardFunctionTable {
        unsafe { &*(self as *const Self as *const OpaqueOsKeyboardFunctionTable) }
    }
}

extern "C" fn c_keyboard<'a, T: 'a + OsKeyboardInner<'a>>(
    os: &'a mut T,
    lib: COptArc<Library>,
    out: &mut MuPluginKeyboard<'a>,
) -> i32 {
    os.keyboard()
        .map(|k| PluginKeyboard::new(k, lib))
        .into_int_out_result(out)
}

extern "C" fn c_into_keyboard<P: 'static + Keyboard + Clone, T: 'static + PluginOsKeyboard<P>>(
    os: &mut T,
    lib: COptArc<Library>,
    out: &mut MuArcPluginKeyboard,
) -> i32 {
    let os = unsafe { Box::from_raw(os) };
    os.into_keyboard()
        .map(|k| ArcPluginKeyboard::new(k, lib))
        .into_int_out_result(out)
}
