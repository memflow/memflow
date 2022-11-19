//! Describes optional keyboard input for a Operating System

use crate::cglue::*;
use crate::prelude::v1::Result;

#[cfg_attr(feature = "plugins", cglue_trait)]
#[int_result]
pub trait OsKeyboard: Send {
    #[wrap_with_obj(crate::os::keyboard::Keyboard)]
    type KeyboardType<'a>: crate::os::keyboard::Keyboard + 'a
    where
        Self: 'a;
    #[wrap_with_group(crate::os::keyboard::IntoKeyboard)]
    type IntoKeyboardType: crate::os::keyboard::Keyboard + Clone + 'static;

    fn keyboard(&mut self) -> Result<Self::KeyboardType<'_>>;
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
