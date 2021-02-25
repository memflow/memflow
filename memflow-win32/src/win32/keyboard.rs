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
use memflow::os::{Keyboard, KeyboardState};
use memflow_win32::win32::{Win32Kernel, Win32Keyboard};

fn test<T: PhysicalMemory, V: VirtualTranslate>(kernel: &mut Win32Kernel<T, V>) {
    let mut kbd = Win32Keyboard::with_kernel_ref(kernel).unwrap();

    loop {
        let kbs = kbd.state().unwrap();
        println!("space down: {:?}", kbs.is_down(0x20)); // VK_SPACE
        thread::sleep(time::Duration::from_millis(1000));
    }
}
```
*/
use super::{Win32Kernel, Win32ProcessInfo, Win32VirtualTranslate};

use memflow::error::{Error, ErrorKind, ErrorOrigin, Result};
use memflow::mem::{PhysicalMemory, VirtualDMA, VirtualMemory, VirtualTranslate};
use memflow::os::{Keyboard, KeyboardState, Process};
use memflow::prelude::OSInner;

use std::convert::TryInto;

use log::debug;

use memflow::error::PartialResultExt;
use memflow::types::Address;

use pelite::{self, pe64::exports::Export, PeView};

/// Interface for accessing the target's keyboard state.
#[derive(Clone, Debug)]
pub struct Win32Keyboard<T> {
    pub virt_mem: T,
    user_process_info: Win32ProcessInfo,
    key_state_addr: Address,
}

impl<'a, T: PhysicalMemory, V: VirtualTranslate>
    Win32Keyboard<VirtualDMA<T, V, Win32VirtualTranslate>>
{
    pub fn with_kernel(mut kernel: Win32Kernel<T, V>) -> Result<Self> {
        let (user_process_info, key_state_addr) = Self::find_keystate(&mut kernel)?;

        let (phys_mem, vat) = kernel.virt_mem.destroy();
        let virt_mem = VirtualDMA::with_vat(
            phys_mem,
            user_process_info.base_info.proc_arch,
            user_process_info.translator(),
            vat,
        );

        Ok(Self {
            virt_mem,
            user_process_info,
            key_state_addr,
        })
    }

    /// Consume the self object and return the underlying owned memory and vat objects
    pub fn destroy(self) -> (T, V) {
        self.virt_mem.destroy()
    }
}

impl<'a, T: PhysicalMemory, V: VirtualTranslate>
    Win32Keyboard<VirtualDMA<&'a mut T, &'a mut V, Win32VirtualTranslate>>
{
    /// Constructs a new keyboard object by borrowing a kernel object.
    ///
    /// Internally this will create a `VirtualDMA` object that also
    /// borrows the PhysicalMemory and Vat objects from the kernel.
    ///
    /// The resulting process object is NOT cloneable due to the mutable borrowing.
    ///
    /// When u need a cloneable Process u have to use the `::with_kernel` function
    /// which will move the kernel object.
    pub fn with_kernel_ref(kernel: &'a mut Win32Kernel<T, V>) -> Result<Self> {
        let (user_process_info, key_state_addr) = Self::find_keystate(kernel)?;

        let (phys_mem, vat) = kernel.virt_mem.mem_vat_pair();
        let virt_mem = VirtualDMA::with_vat(
            phys_mem,
            user_process_info.base_info.proc_arch,
            user_process_info.translator(),
            vat,
        );

        Ok(Self {
            virt_mem,
            user_process_info,
            key_state_addr,
        })
    }
}

impl<T> Win32Keyboard<T> {
    fn find_keystate<P: PhysicalMemory, V: VirtualTranslate>(
        kernel: &mut Win32Kernel<P, V>,
    ) -> Result<(Win32ProcessInfo, Address)> {
        let win32kbase_module_info = kernel.module_by_name("win32kbase.sys")?;
        debug!("found win32kbase.sys: {:?}", win32kbase_module_info);

        let user_process_info = kernel
            .process_info_by_name("winlogon.exe")
            .or_else(|_| kernel.process_info_by_name("wininit.exe"))
            .or_else(|_| kernel.process_info_by_name("explorer.exe"))?;
        let user_process_info_win32 =
            kernel.process_info_from_base_info(user_process_info.clone())?;
        let mut user_process = kernel.process_by_info(user_process_info)?;
        debug!("found user proxy process: {:?}", user_process);

        // read with user_process dtb
        let module_buf = user_process
            .virt_mem()
            .virt_read_raw(win32kbase_module_info.base, win32kbase_module_info.size)
            .data_part()?;
        debug!("fetched {:x} bytes from win32kbase.sys", module_buf.len());

        // TODO: lazy
        let export_addr =
            Self::find_gaf_pe(&module_buf).or_else(|_| Self::find_gaf_sig(&module_buf))?;

        Ok((
            user_process_info_win32,
            win32kbase_module_info.base + export_addr,
        ))
    }

    fn find_gaf_pe(module_buf: &[u8]) -> Result<usize> {
        let pe = PeView::from_bytes(module_buf)
            .map_err(|err| Error(ErrorOrigin::OSLayer, ErrorKind::InvalidPeFile).log_info(err))?;

        match pe.get_export_by_name("gafAsyncKeyState").map_err(|_| {
            Error(ErrorOrigin::OSLayer, ErrorKind::ExportNotFound)
                .log_info("unable to find gafAsyncKeyState: {}")
        })? {
            Export::Symbol(s) => {
                debug!("gafAsyncKeyState export found at: {:x}", *s);
                Ok(*s as usize)
            }
            Export::Forward(_) => Err(Error(ErrorOrigin::OSLayer, ErrorKind::ExportNotFound)
                .log_info("export gafAsyncKeyState found but it is forwarded")),
        }
    }

    // TODO: replace with a custom signature scanning crate
    #[cfg(feature = "regex")]
    fn find_gaf_sig(module_buf: &[u8]) -> Result<usize> {
        use ::regex::bytes::*;

        // 48 8B 05 ? ? ? ? 48 89 81 ? ? 00 00 48 8B 8F + 0x3
        let re = Regex::new("(?-u)\\x48\\x8B\\x05(?s:.)(?s:.)(?s:.)(?s:.)\\x48\\x89\\x81(?s:.)(?s:.)\\x00\\x00\\x48\\x8B\\x8F")
                    .map_err(|_| Error(ErrorOrigin::OSLayer, ErrorKind::Encoding).log_info("malformed gafAsyncKeyState signature"))?;
        let buf_offs = re
            .find(&module_buf[..])
            .ok_or_else(|| {
                Error(ErrorOrigin::OSLayer, ErrorKind::NotFound)
                    .log_info("unable to find gafAsyncKeyState signature")
            })?
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
        Err(
            Error(ErrorOrigin::OSLayer, ErrorKind::UnsupportedOptionalFeature)
                .log_error("signature scanning requires std"),
        )
    }
}

impl<T> Keyboard for Win32Keyboard<T>
where
    T: VirtualMemory,
{
    type KeyboardStateType = Win32KeyboardState;

    /// Reads the gafAsyncKeyState global from the win32kbase.sys kernel module.
    fn state(&mut self) -> memflow::error::Result<Self::KeyboardStateType> {
        let buffer: [u8; 256 * 2 / 8] = self.virt_mem.virt_read(self.key_state_addr)?;
        Ok(Win32KeyboardState { buffer })
    }

    /// Writes the gafAsyncKeyState global to the win32kbase.sys kernel module.
    ///
    /// # Remarks:
    ///
    /// This will not enforce key presses in all applications on Windows.
    /// It will only modify calls to GetKeyState / GetAsyncKeyState.
    fn set_state(&mut self, state: &Self::KeyboardStateType) -> memflow::error::Result<()> {
        self.virt_mem
            .virt_write(self.key_state_addr, &state.buffer)
            .data_part()
    }
}

/// Represents the current Keyboardstate.
///
/// Internally this will hold a 256 * 2 / 8 byte long copy of the gafAsyncKeyState array from the target.
#[derive(Clone)]
pub struct Win32KeyboardState {
    buffer: [u8; 256 * 2 / 8],
}

macro_rules! get_ks_byte {
    ($vk:expr) => {
        $vk * 2 / 8
    };
}

macro_rules! get_ks_down_bit {
    ($vk:expr) => {
        1 << (($vk % 4) * 2)
    };
}

macro_rules! is_key_down {
    ($ks:expr, $vk:expr) => {
        ($ks[get_ks_byte!($vk) as usize] & get_ks_down_bit!($vk)) != 0
    };
}

macro_rules! set_key_down {
    ($ks:expr, $vk:expr, $down:expr) => {
        if $down {
            ($ks[get_ks_byte!($vk) as usize] |= get_ks_down_bit!($vk))
        } else {
            ($ks[get_ks_byte!($vk) as usize] &= !get_ks_down_bit!($vk))
        }
    };
}

impl KeyboardState for Win32KeyboardState {
    /// Returns true wether the given key was pressed.
    /// This function accepts a valid microsoft virtual keycode.
    ///
    /// A list of all Keycodes can be found on the [msdn](https://docs.microsoft.com/en-us/windows/win32/inputdev/virtual-key-codes).
    ///
    /// In case of supplying a invalid key this function will just return false cleanly.
    fn is_down(&self, vk: i32) -> bool {
        if !(0..=256).contains(&vk) {
            false
        } else {
            is_key_down!(self.buffer, vk)
        }
    }

    fn set_down(&mut self, vk: i32, down: bool) {
        if (0..=256).contains(&vk) {
            set_key_down!(self.buffer, vk, down);
        }
    }
}
