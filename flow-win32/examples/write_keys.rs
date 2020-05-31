use std::{thread, time};

use log::Level;

use flow_core::*;
use flow_win32::*;

pub fn write_string<T: VirtualMemory>(
    mem: &mut T,
    kbd: &Keyboard,
    input: &[i32],
) -> flow_core::Result<()> {
    for &k in input.iter() {
        let mut kbs = kbd.state(mem)?;
        kbs.set_down(k, true);
        kbd.set_state(mem, &kbs)?;
        thread::sleep(time::Duration::from_millis(100));
        kbs.set_down(k, false);
        kbd.set_state(mem, &kbs)?;
        thread::sleep(time::Duration::from_millis(100));
    }
    Ok(())
}

pub fn main() {
    simple_logger::init_with_level(Level::Debug).unwrap();

    let mut mem = flow_qemu_procfs::Memory::new().unwrap();

    let win = Win32::try_with(&mut mem).unwrap();
    let offsets = Win32Offsets::try_with_guid(&win.kernel_guid()).unwrap();

    let kbd = Keyboard::with(&mut mem, &win, &offsets).unwrap();

    write_string(
        &mut mem,
        &kbd,
        &[
            win_key_codes::VK_H,
            win_key_codes::VK_E,
            win_key_codes::VK_L,
            win_key_codes::VK_L,
            win_key_codes::VK_O,
            win_key_codes::VK_SPACE,
            win_key_codes::VK_W,
            win_key_codes::VK_O,
            win_key_codes::VK_R,
            win_key_codes::VK_L,
            win_key_codes::VK_D,
        ],
    )
    .unwrap();
}
