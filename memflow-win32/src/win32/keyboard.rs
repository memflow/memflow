/*!
Module for reading a target's keyboard state.

The `gafAsyncKeyState` array contains the current Keyboard state on Windows targets.
This array will internally be read by the [`GetAsyncKeyState()`](https://docs.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-getasynckeystate) function of Windows.

Although the gafAsyncKeyState array is exported by the win32kbase.sys kernel module it is only properly mapped into user mode processes.
Therefor the Keyboard will by default find the winlogon.exe or wininit.exe process and use it as a proxy to read the data.

# Examples:

```
use std::{thread, time};

use memflow::mem::{PhysicalMemory, VirtualTranslate};
use memflow_win32::win32::{Kernel, Keyboard};

fn test<T: PhysicalMemory, V: VirtualTranslate>(kernel: &mut Kernel<T, V>) {
    let kbd = Keyboard::try_with(kernel).unwrap();

    loop {
        let kbs = kbd.state_with_kernel(kernel).unwrap();
        println!("space down: {:?}", kbs.is_down(win_key_codes::VK_SPACE));
        thread::sleep(time::Duration::from_millis(1000));
    }
}
```
*/
use super::{Kernel, Win32Process, Win32ProcessInfo};
use crate::error::{Error, Result};

use std::convert::TryInto;

use log::debug;

use memflow::error::PartialResultExt;
use memflow::mem::{PhysicalMemory, VirtualMemory, VirtualTranslate};
use memflow::process::OsProcessModuleInfo;
use memflow::types::Address;

use pelite::{self, pe64::exports::Export, PeView};

/// Interface for accessing the target's keyboard state.
#[derive(Clone, Debug)]
pub struct Keyboard {
    user_process_info: Win32ProcessInfo,
    key_state_addr: Address,
}

/// Represents the current Keyboardstate.
///
/// Internally this will hold a 256 * 2 / 8 byte long copy of the gafAsyncKeyState array from the target.
#[derive(Clone)]
pub struct KeyboardState {
    buffer: [u8; 256 * 2 / 8],
}

impl Keyboard {
    pub fn try_with<T: PhysicalMemory, V: VirtualTranslate>(
        kernel: &mut Kernel<T, V>,
    ) -> Result<Self> {
        let kernel_process_info = kernel.kernel_process_info()?;
        debug!("found ntoskrnl.exe: {:?}", kernel_process_info);

        let win32kbase_module_info = {
            let mut ntoskrnl_process = Win32Process::with_kernel_ref(kernel, kernel_process_info);
            ntoskrnl_process.module_info("win32kbase.sys")?
        };
        debug!("found win32kbase.sys: {:?}", win32kbase_module_info);

        let user_process_info = kernel
            .process_info("winlogon.exe")
            .or_else(|_| kernel.process_info("wininit.exe"))?;
        let mut user_process = Win32Process::with_kernel_ref(kernel, user_process_info.clone());
        debug!("found user proxy process: {:?}", user_process);

        // read with user_process dtb
        let module_buf = user_process
            .virt_mem
            .virt_read_raw(win32kbase_module_info.base(), win32kbase_module_info.size())
            .data_part()?;
        debug!("fetched {:x} bytes from win32kbase.sys", module_buf.len());

        // TODO: lazy
        let export_addr =
            Self::find_gaf_pe(&module_buf).or_else(|_| Self::find_gaf_sig(&module_buf))?;

        Ok(Self {
            user_process_info,
            key_state_addr: win32kbase_module_info.base() + export_addr,
        })
    }

    /// Fetches the gafAsyncKeyState from the given virtual reader.
    /// This will use the given virtual memory reader to fetch
    /// the gafAsyncKeyState from the win32kbase.sys kernel module.
    pub fn state<T: VirtualMemory>(&self, virt_mem: &mut T) -> Result<KeyboardState> {
        let buffer: [u8; 256 * 2 / 8] = virt_mem.virt_read(self.key_state_addr)?;
        Ok(KeyboardState { buffer })
    }

    /// Fetches the kernel's gafAsyncKeyState state with the kernel context.
    /// This will use the winlogon.exe or wininit.exe process as a proxy for reading
    /// the gafAsyncKeyState from the win32kbase.sys kernel module.
    pub fn state_with_kernel<T: PhysicalMemory, V: VirtualTranslate>(
        &self,
        kernel: &mut Kernel<T, V>,
    ) -> Result<KeyboardState> {
        let mut user_process =
            Win32Process::with_kernel_ref(kernel, self.user_process_info.clone());
        self.state(&mut user_process.virt_mem)
    }

    /// Fetches the kernel's gafAsyncKeyState state with a processes context.
    /// The win32kbase.sys kernel module is accessible with the DTB of a user process
    /// so any usermode process can be used to read this memory region.
    pub fn state_with_process<T: VirtualMemory>(
        &self,
        process: &mut Win32Process<T>,
    ) -> Result<KeyboardState> {
        self.state(&mut process.virt_mem)
    }

    fn find_gaf_pe(module_buf: &[u8]) -> Result<usize> {
        let pe = PeView::from_bytes(module_buf).map_err(Error::from)?;

        match pe
            .get_export_by_name("gafAsyncKeyState")
            .map_err(Error::from)?
        {
            Export::Symbol(s) => {
                debug!("gafAsyncKeyState export found at: {:x}", *s);
                Ok(*s as usize)
            }
            Export::Forward(_) => Err(Error::Other(
                "export gafAsyncKeyState found but it is forwarded",
            )),
        }
    }

    // TODO: replace with a custom signature scanning crate
    #[cfg(feature = "regex")]
    fn find_gaf_sig(module_buf: &[u8]) -> Result<usize> {
        use ::regex::bytes::*;

        // 48 8B 05 ? ? ? ? 48 89 81 ? ? 00 00 48 8B 8F + 0x3
        let re = Regex::new("(?-u)\\x48\\x8B\\x05(?s:.)(?s:.)(?s:.)(?s:.)\\x48\\x89\\x81(?s:.)(?s:.)\\x00\\x00\\x48\\x8B\\x8F")
                    .map_err(|_| Error::Other("malformed gafAsyncKeyState signature"))?;
        let buf_offs = re
            .find(&module_buf[..])
            .ok_or_else(|| Error::Other("unable to find gafAsyncKeyState signature"))?
            .start()
            + 0x3;

        // compute rip relative addr
        let export_offs = buf_offs as u32
            + u32::from_le_bytes(module_buf[buf_offs..buf_offs + 4].try_into().unwrap())
            + 0x4;
        debug!("gafAsyncKeyState export found at: {:x}", export_offs);
        Ok(export_offs as usize)
    }

    #[cfg(not(feature = "regex"))]
    fn find_gaf_sig(module_buf: &[u8]) -> Result<usize> {
        Err(Error::Other("signature scanning requires std"))
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

//#define SET_KEY_LOCKED(ks, vk, down) (ks)[GET_KS_BYTE(vk)] = ((down) ? \
//                                                              ((ks)[GET_KS_BYTE(vk)] | GET_KS_LOCK_BIT(vk)) : \
//                                                              ((ks)[GET_KS_BYTE(vk)] & ~GET_KS_LOCK_BIT(vk)))

impl KeyboardState {
    /// Returns true wether the given key was pressed.
    /// This function accepts a valid microsoft virtual keycode.
    ///
    /// A list of all Keycodes can be found on the [msdn](https://docs.microsoft.com/en-us/windows/win32/inputdev/virtual-key-codes).
    ///
    /// In case of supplying a invalid key this function will just return false cleanly.
    pub fn is_down(&self, vk: i32) -> bool {
        if vk < 0 || vk > 256 {
            false
        } else {
            is_key_down!(self.buffer, vk)
        }
    }
}
