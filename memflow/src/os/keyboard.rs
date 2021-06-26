//! Describes optional keyboard input for a Operating System

use crate::cglue::*;
use crate::prelude::v1::Result;

/// OsKeyboard supertrait for all possible lifetimes
///
/// Use this for convenience. Chances are, once GAT are implemented, only `OsKeyboard` will be kept.
///
/// It naturally provides all `OsKeyboardInner` functions.
pub trait OsKeyboard: for<'a> OsKeyboardInner<'a> {}
impl<T: for<'a> OsKeyboardInner<'a>> OsKeyboard for T {}

#[cfg_attr(feature = "plugins", cglue_trait)]
#[int_result]
pub trait OsKeyboardInner<'a>: Send {
    #[wrap_with_obj(crate::os::keyboard::Keyboard)]
    type KeyboardType: crate::os::keyboard::Keyboard + 'a;
    #[wrap_with_group(crate::os::keyboard::IntoKeyboard)]
    type IntoKeyboardType: crate::os::keyboard::Keyboard + 'static;

    fn keyboard(&'a mut self) -> Result<Self::KeyboardType>;
    fn into_keyboard(self) -> Result<Self::IntoKeyboardType>;
}

#[cfg(feature = "plugins")]
cglue_trait_group!(IntoKeyboard, { Keyboard, Clone }, {});

#[cfg_attr(feature = "plugins", cglue_trait)]
#[int_result]
#[cglue_forward]
pub trait Keyboard {
    #[wrap_with_obj(crate::os::keyboard::KeyboardState)]
    type KeyboardStateType: crate::os::keyboard::KeyboardState;

    fn is_down(&mut self, vk: i32) -> bool;
    fn set_down(&mut self, vk: i32, down: bool);

    fn state(&mut self) -> Result<Self::KeyboardStateType>;
}

#[cfg_attr(feature = "plugins", cglue_trait)]
#[int_result]
#[cglue_forward]
pub trait KeyboardState {
    fn is_down(&self, vk: i32) -> bool;
}
