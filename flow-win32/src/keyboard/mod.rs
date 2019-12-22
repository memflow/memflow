// let kbd = Keyboard::with(mem, ...)
// let ks = kbd.keystate()
// ks.get(VK_A)
// kbd.set_keystate(...)

use crate::error::Result;
use crate::win::Windows;

//use flow_core::address::{Address, Length};
use flow_core::mem::*;

use crate::win::process::*;

pub struct Keyboard {
    // kernel.read(key_state_addr)
//key_state_addr: Address,
}

pub struct KeyState {}

impl Keyboard {
    pub fn with<T: VirtualRead>(win: &Windows<T>) -> Result<Self> {
        let kernel = win.kernel_process()?;
        let _kbase = kernel.module("win32kbase.sys")?;
        Ok(Self {})
    }
}
