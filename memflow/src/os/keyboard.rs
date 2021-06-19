//! Describes optional keyboard input for a Operating System

use crate::prelude::v1::Result;
use cglue::prelude::v1::*;

/// OsKeyboard supertrait for all possible lifetimes
///
/// Use this for convenience. Chances are, once GAT are implemented, only `OsKeyboard` will be kept.
///
/// It naturally provides all `OsKeyboardInner` functions.
pub trait OsKeyboard: for<'a> OsKeyboardInner<'a> {}
impl<T: for<'a> OsKeyboardInner<'a>> OsKeyboard for T {}

#[cglue_trait]
#[int_result]
//#[cglue_forward]
pub trait OsKeyboardInner<'a>: Send {
    #[wrap_with_obj(crate::os::keyboard::Keyboard)]
    type KeyboardType: crate::os::keyboard::Keyboard + 'a;
    #[wrap_with_obj(crate::os::keyboard::Keyboard)]
    type IntoKeyboardType: crate::os::keyboard::Keyboard + 'static;

    fn keyboard(&'a mut self) -> Result<Self::KeyboardType>;
    fn into_keyboard(self) -> Result<Self::IntoKeyboardType>;
}

#[cglue_trait]
#[int_result]
pub trait Keyboard {
    fn is_down(&mut self, vk: i32) -> bool;
    fn set_down(&mut self, vk: i32, down: bool);
}
