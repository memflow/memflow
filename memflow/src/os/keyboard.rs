//! Describes optional keyboard input for a Operating System

use crate::prelude::v1::Result;

/// OSKeyboard supertrait for all possible lifetimes
///
/// Use this for convenience. Chances are, once GAT are implemented, only `OSKeyboard` will be kept.
///
/// It naturally provides all `OSKeyboardInner` functions.
pub trait OSKeyboard: for<'a> OSKeyboardInner<'a> {}
impl<T: for<'a> OSKeyboardInner<'a>> OSKeyboard for T {}

pub trait OSKeyboardInner<'a>: Send {
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
