use std::{thread, time};

use clap::*;
use log::Level;

use memflow::connector::*;

use memflow_win32::win32::{Kernel, Keyboard};

pub fn main() {
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
    let level = match matches.occurrences_of("verbose") {
        0 => Level::Error,
        1 => Level::Warn,
        2 => Level::Info,
        3 => Level::Debug,
        4 => Level::Trace,
        _ => Level::Trace,
    };
    simple_logger::SimpleLogger::new()
        .with_level(level.to_level_filter())
        .init()
        .unwrap();

    // create inventory + connector
    let inventory = unsafe { ConnectorInventory::scan() };
    let connector = unsafe {
        inventory.create_connector(
            matches.value_of("connector").unwrap(),
            &ConnectorArgs::parse(matches.value_of("args").unwrap()).unwrap(),
        )
    }
    .unwrap();

    // creating the kernel object
    let mut kernel = Kernel::builder(connector)
        .build_default_caches()
        .build()
        .unwrap();

    // fetch keyboard state
    let kbd = Keyboard::try_with(&mut kernel).unwrap();

    loop {
        let kbs = kbd.state_with_kernel(&mut kernel).unwrap();
        println!("space down: {:?}", kbs.is_down(win_key_codes::VK_SPACE));
        thread::sleep(time::Duration::from_millis(1000));
    }
}
