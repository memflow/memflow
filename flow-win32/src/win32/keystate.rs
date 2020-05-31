use super::{Win32, Win32Module, Win32Offsets, Win32Process};
use crate::error::{Error, Result};

use flow_core::mem::VirtualMemory;
use flow_core::process::{OsProcess, OsProcessModule};
use flow_core::{Address, Length};

use log::debug;

use pelite::{self, pe64::exports::Export, PeView};

pub struct Keyboard {
    user_process: Win32Process,
    key_state_addr: Address,
}

pub struct KeyboardState {
    buffer: Box<[u8; 256 * 2 / 8]>,
}

impl Keyboard {
    pub fn with<T: VirtualMemory>(
        mem: &mut T,
        win: &Win32,
        offsets: &Win32Offsets,
    ) -> Result<Self> {
        let kernel_process = Win32Process::try_from_kernel(mem, win)?;
        debug!("found kernel_process: {:?}", kernel_process);
        let kernel_module =
            Win32Module::try_with_name(mem, &kernel_process, offsets, "win32kbase.sys")?;
        debug!("found kernel_module: {:?}", kernel_module);

        let user_process = Win32Process::try_with_name(mem, win, offsets, "winlogon.exe")
            .or_else(|_| Win32Process::try_with_name(mem, win, offsets, "wininit.exe"))?;
        debug!("found user_process: {:?}", user_process);

        // read with user_process dtb
        let mut virt_mem = user_process.virt_mem(mem);

        let module_buf = virt_mem.virt_read_raw(kernel_module.base(), kernel_module.size())?;
        let pe = PeView::from_bytes(&module_buf).map_err(Error::new)?;
        let export_addr = match pe
            .get_export_by_name("gafAsyncKeyState")
            .map_err(Error::new)?
        {
            Export::Symbol(s) => kernel_module.base() + Length::from(*s),
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

    pub fn state<T: VirtualMemory>(&self, mem: &mut T) -> Result<KeyboardState> {
        let mut virt_mem = self.user_process.virt_mem(mem);
        let buffer: [u8; 256 * 2 / 8] = virt_mem.virt_read(self.key_state_addr)?;
        Ok(KeyboardState {
            buffer: Box::new(buffer),
        })
    }

    pub fn set_state<T: VirtualMemory>(&self, mem: &mut T, state: &KeyboardState) -> Result<()> {
        let mut virt_mem = self.user_process.virt_mem(mem);
        virt_mem.virt_write(self.key_state_addr, &*state.buffer)?;
        Ok(())
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

// #define IS_KEY_LOCKED(ks, vk) (((ks)[GET_KS_BYTE(vk)] & GET_KS_LOCK_BIT(vk)) ? TRUE : FALSE)

// #define SET_KEY_DOWN(ks, vk, down) (ks)[GET_KS_BYTE(vk)] = ((down) ? \
//                                                              ((ks)[GET_KS_BYTE(vk)] | GET_KS_DOWN_BIT(vk)) : \
//                                                              ((ks)[GET_KS_BYTE(vk)] & ~GET_KS_DOWN_BIT(vk)))
macro_rules! set_key_down {
    ($ks:expr, $vk:expr) => {
        $ks[get_ks_byte!($vk) as usize] = $ks[get_ks_byte!($vk) as usize] | get_ks_down_bit!($vk)
    };
}
macro_rules! set_key_up {
    ($ks:expr, $vk:expr) => {
        $ks[get_ks_byte!($vk) as usize] = $ks[get_ks_byte!($vk) as usize] & !get_ks_down_bit!($vk)
    };
}

//#define SET_KEY_LOCKED(ks, vk, down) (ks)[GET_KS_BYTE(vk)] = ((down) ? \
//                                                              ((ks)[GET_KS_BYTE(vk)] | GET_KS_LOCK_BIT(vk)) : \
//                                                              ((ks)[GET_KS_BYTE(vk)] & ~GET_KS_LOCK_BIT(vk)))

impl KeyboardState {
    pub fn down(&self, vk: i32) -> Result<bool> {
        if vk < 0 || vk > 256 {
            Err(Error::new("invalid key"))
        } else {
            Ok(is_key_down!(self.buffer, vk))
        }
    }

    pub fn set_down(&mut self, vk: i32, down: bool) {
        if down {
            set_key_down!(self.buffer, vk);
        } else {
            set_key_up!(self.buffer, vk);
        }
    }
}
