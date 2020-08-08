use clap::*;
use log::Level;
use std::fs::File;
use std::io::Write;

use memflow_core::connector::*;

use memflow_win32::{Kernel, Win32OffsetsFile};

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
        .arg(
            Arg::with_name("output")
                .long("output")
                .short("o")
                .takes_value(true),
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
    let connector = unsafe {
        inventory.create_connector(
            matches.value_of("connector").unwrap(),
            &ConnectorArgs::try_parse_str(matches.value_of("args").unwrap()).unwrap(),
        )
    }
    .unwrap();

    let kernel = Kernel::builder(connector)
        .build_default_caches()
        .build()
        .unwrap();

    // write offsets to file
    let offsets = toml::to_string_pretty(&Win32OffsetsFile {
        kernel_guid: kernel.kernel_info.kernel_guid,
        kernel_winver: kernel.kernel_info.kernel_winver,
        offsets: kernel.offsets,
    })
    .unwrap();
    match matches.value_of("output") {
        Some(output) => {
            let mut file = File::create(output).unwrap();
            file.write_all(offsets.as_bytes()).unwrap();
        }
        None => println!("{}", offsets),
    }
}
