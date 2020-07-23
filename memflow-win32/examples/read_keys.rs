use log::Level;
use std::{thread, time};

use memflow_core::mem::TranslateArch;

use memflow_connector::create_connector;

use memflow_win32::error::Result;
use memflow_win32::offsets::Win32Offsets;
use memflow_win32::win32::{Kernel, KernelInfo, Keyboard};

pub fn main() -> Result<()> {
    simple_logger::init_with_level(Level::Debug).unwrap();

    let mut mem_sys = create_connector("")?;
    let kernel_info = KernelInfo::scanner().mem(&mut mem_sys).scan()?;

    let vat = TranslateArch::new(kernel_info.start_block.arch);
    let offsets = Win32Offsets::try_with_kernel_info(&kernel_info)?;
    let mut kernel = Kernel::new(mem_sys, vat, offsets, kernel_info);

    let kbd = Keyboard::try_with(&mut kernel)?;

    loop {
        let kbs = kbd.state_with_kernel(&mut kernel)?;
        println!("{:?}", kbs.is_down(win_key_codes::VK_SPACE));
        thread::sleep(time::Duration::from_millis(1000));
    }
}
