use std::{thread, time};

use clap::*;
use log::Level;

use memflow_core::connector::*;
use memflow_core::mem::{CachedMemoryAccess, CachedVirtualTranslate, TranslateArch};

use memflow_win32::error::Result;
use memflow_win32::offsets::Win32Offsets;
use memflow_win32::win32::{Kernel, KernelInfo, Keyboard};

pub fn main() -> Result<()> {
    let matches = App::new("read_keys example")
        .version(crate_version!())
        .author(crate_authors!())
        .arg(Arg::with_name("verbose").short("v").multiple(true))
        .arg(
            Arg::with_name("connector")
                .long("connector")
                .short("c")
                .takes_value(true)
                .required(true),
        )
        .arg(
            Arg::with_name("args")
                .long("args")
                .short("a")
                .takes_value(true)
                .default_value(""),
        )
        .get_matches();

    // set log level
    match matches.occurrences_of("verbose") {
        1 => simple_logger::init_with_level(Level::Warn).unwrap(),
        2 => simple_logger::init_with_level(Level::Info).unwrap(),
        3 => simple_logger::init_with_level(Level::Debug).unwrap(),
        4 => simple_logger::init_with_level(Level::Trace).unwrap(),
        _ => simple_logger::init_with_level(Level::Error).unwrap(),
    }

    // create inventory + connector
    let inventory = unsafe { ConnectorInventory::try_new() }.unwrap();
    let mut connector = unsafe {
        inventory.create_connector(
            matches.value_of("connector").unwrap(),
            &ConnectorArgs::try_parse_str(matches.value_of("args").unwrap()).unwrap(),
        )
    }
    .unwrap();

    // scan for win32 kernel
    let kernel_info = KernelInfo::scanner(&mut *connector).scan()?;
    let offsets = Win32Offsets::try_with_kernel_info(&kernel_info)?;

    // TODO: builder testing
    let vat = TranslateArch::new(kernel_info.start_block.arch);

    let mut connector_cached = CachedMemoryAccess::builder(&mut connector)
        .arch(kernel_info.start_block.arch)
        .build()
        .unwrap();

    let vat_cached = CachedVirtualTranslate::builder(vat)
        .arch(kernel_info.start_block.arch)
        .build()
        .unwrap();

    let mut kernel = Kernel::new(&mut connector_cached, vat_cached, offsets, kernel_info);

    // fetch keyboard state
    let kbd = Keyboard::try_with(&mut kernel)?;

    loop {
        let kbs = kbd.state_with_kernel(&mut kernel)?;
        println!("space down: {:?}", kbs.is_down(win_key_codes::VK_SPACE));
        thread::sleep(time::Duration::from_millis(1000));
    }
}
