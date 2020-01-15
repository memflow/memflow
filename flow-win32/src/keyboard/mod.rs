use crate::error::{Error, Result};
use crate::win32::*;

use flow_core::address::{Address, Length};
use flow_core::mem::*;
use flow_core::*;

use pelite::{self, pe64::exports::Export, PeView};

pub struct Keyboard {
    user_process: Win32UserProcess,
    key_state_addr: Address,
}

pub struct KeyboardState {
    buffer: Vec<u8>,
}

impl Keyboard {
    pub fn with(win: &Win32) -> Result<Self> {
        let kernel_process = win.kernel_process()?;
        let mut kernel_module = kernel_process.module("win32kbase.sys")?;

        // TODO: helper object with multiple tries
        let mut user_process = match win.process("winlogon.exe") {
            Ok(p) => p,
            Err(_) => win.process("wininit.exe")?,
        };

        let base = kernel_module.base()?;
        let size = kernel_module.size()?;

        let buf = user_process.virt_read(base, size)?;
        let pe = PeView::from_bytes(&buf)?;
        let export_addr = match pe.get_export_by_name("gafAsyncKeyState")? {
            Export::Symbol(s) => base + Length::from(*s),
            Export::Forward(_) => {
                return Err(Error::new(
                    "export gafAsyncKeyState found but it is forwarded",
                ))
            }
        };

        Ok(Self {
            user_process,
            key_state_addr: export_addr,
        })
    }

    pub fn state(&mut self) -> Result<KeyboardState> {
        let buffer = self
            .user_process
            .virt_read(self.key_state_addr, len!(256 * 2 / 8))?;
        Ok(KeyboardState { buffer })
    }
}

// #define GET_KS_BYTE(vk) ((vk)*2 / 8)
macro_rules! get_ks_byte {
    ($vk:expr) => {
        $vk * 2 / 8
    };
}

// #define GET_KS_DOWN_BIT(vk) (1 << (((vk) % 4) * 2))
macro_rules! get_ks_down_bit {
    ($vk:expr) => {
        1 << (($vk % 4) * 2)
    };
}

// #define IS_KEY_DOWN(ks, vk) (((ks)[GET_KS_BYTE(vk)] & GET_KS_DOWN_BIT(vk)) ? true : false)
macro_rules! is_key_down {
    ($ks:expr, $vk:expr) => {
        ($ks[get_ks_byte!($vk) as usize] & get_ks_down_bit!($vk)) != 0
    };
}

impl KeyboardState {
    pub fn down(&self, vk: i32) -> Result<bool> {
        if vk < 0 || vk > 256 {
            Err(Error::new("invalid key"))
        } else {
            Ok(is_key_down!(self.buffer, vk))
        }
    }
}
