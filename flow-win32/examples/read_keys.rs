use log::Level;
use simple_logger;
use std::{thread, time};

use flow_core::error::Result;
use flow_core::mem::TranslateArch;

use flow_qemu_procfs::Memory;

use flow_win32::offsets::Win32Offsets;
use flow_win32::win32::{Kernel, KernelInfo, Keyboard};

pub fn main() -> Result<()> {
    simple_logger::init_with_level(Level::Debug).unwrap();

    let mut mem_sys = Memory::new()?;
    let kernel_info = KernelInfo::find(&mut mem_sys)?;

    let vat = TranslateArch::new(kernel_info.start_block.arch);
    let offsets = Win32Offsets::try_with_guid(&kernel_info.kernel_guid)?;
    let mut kernel = Kernel::new(mem_sys, vat, offsets, kernel_info);

    let kbd = Keyboard::try_with(&mut kernel)?;

    loop {
        let kbs = kbd.state_with_kernel(&mut kernel)?;
        println!("{:?}", kbs.is_down(win_key_codes::VK_SPACE));
        thread::sleep(time::Duration::from_millis(1000));
    }
}
