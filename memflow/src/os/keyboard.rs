//! Describes optional keyboard input for a Operating System

use crate::prelude::v1::Result;

/// OsKeyboard supertrait for all possible lifetimes
///
/// Use this for convenience. Chances are, once GAT are implemented, only `OsKeyboard` will be kept.
///
/// It naturally provides all `OsKeyboardInner` functions.
pub trait OsKeyboard: for<'a> OsKeyboardInner<'a> {}
impl<T: for<'a> OsKeyboardInner<'a>> OsKeyboard for T {}

pub trait OsKeyboardInner<'a>: Send {
    type KeyboardType: Keyboard + 'a;
    type IntoKeyboardType: Keyboard;

    fn keyboard(&'a mut self) -> Result<Self::KeyboardType>;
    fn into_keyboard(self) -> Result<Self::IntoKeyboardType>;
}

pub trait Keyboard {
    type KeyboardStateType: KeyboardState + 'static;

    fn state(&mut self) -> Result<Self::KeyboardStateType>;
    fn set_state(&mut self, state: &Self::KeyboardStateType) -> Result<()>;
}

pub trait KeyboardState: Clone {
    fn is_down(&self, vk: i32) -> bool;
    fn set_down(&mut self, vk: i32, down: bool);
}
