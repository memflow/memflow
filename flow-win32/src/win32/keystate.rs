use super::{Kernel, Win32Process, Win32ProcessInfo};
use crate::error::{Error, Result};

use log::debug;

use flow_core::mem::{PhysicalMemory, VirtualMemory, VirtualTranslate};
use flow_core::process::OsProcessModuleInfo;
use flow_core::types::{Address, Length};

use pelite::{self, pe64::exports::Export, PeView};

pub struct Keyboard {
    user_process_info: Win32ProcessInfo,
    key_state_addr: Address,
}

pub struct KeyboardState {
    buffer: [u8; 256 * 2 / 8],
}

impl Keyboard {
    pub fn with<T: PhysicalMemory, V: VirtualTranslate>(kernel: &mut Kernel<T, V>) -> Result<Self> {
        let ntoskrnl_process_info = kernel.ntoskrnl_process_info()?;
        debug!("found ntoskrnl.exe: {:?}", ntoskrnl_process_info);

        let win32kbase_module_info = {
            let mut ntoskrnl_process = Win32Process::with_kernel(kernel, ntoskrnl_process_info);
            ntoskrnl_process.module_info("win32kbase.sys")?
        };
        debug!("found win32kbase.sys: {:?}", win32kbase_module_info);

        let user_process_info = kernel
            .process_info("winlogon.exe")
            .or_else(|_| kernel.process_info("wininit.exe"))?;
        let mut user_process = Win32Process::with_kernel(kernel, user_process_info.clone());
        debug!("found user proxy process: {:?}", user_process);

        // read with user_process dtb
        let module_buf = user_process
            .virt_mem
            .virt_read_raw(win32kbase_module_info.base(), win32kbase_module_info.size())?;

        // TODO: lazy
        let pe = PeView::from_bytes(&module_buf).map_err(Error::new)?;
        let export_addr = match pe
            .get_export_by_name("gafAsyncKeyState")
            .map_err(Error::new)?
        {
            Export::Symbol(s) => win32kbase_module_info.base() + Length::from(*s),
            Export::Forward(_) => {
                return Err(Error::new(
                    "export gafAsyncKeyState found but it is forwarded",
                ))
            }
        };

        Ok(Self {
            user_process_info,
            key_state_addr: export_addr,
        })
    }

    pub fn state<T: PhysicalMemory, V: VirtualTranslate>(
        &self,
        kernel: &mut Kernel<T, V>,
    ) -> Result<KeyboardState> {
        let mut user_process = Win32Process::with_kernel(kernel, self.user_process_info.clone());
        let buffer: [u8; 256 * 2 / 8] = user_process.virt_mem.virt_read(self.key_state_addr)?;
        Ok(KeyboardState { buffer })
    }

    pub fn set_state<T: PhysicalMemory, V: VirtualTranslate>(
        &self,
        kernel: &mut Kernel<T, V>,
        state: &KeyboardState,
    ) -> Result<()> {
        let mut user_process = Win32Process::with_kernel(kernel, self.user_process_info.clone());
        user_process
            .virt_mem
            .virt_write(self.key_state_addr, &state.buffer)?;
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
    pub fn is_down(&self, vk: i32) -> bool {
        if vk < 0 || vk > 256 {
            false
        } else {
            is_key_down!(self.buffer, vk)
        }
    }

    pub fn set_key_down(&mut self, vk: i32) {
        set_key_down!(self.buffer, vk);
    }

    pub fn set_key_up(&mut self, vk: i32) {
        set_key_up!(self.buffer, vk);
    }
}
