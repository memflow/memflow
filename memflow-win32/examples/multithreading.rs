use std::{thread, time};

use clap::*;
use log::Level;

use memflow_core::connector::*;

use memflow_win32::win32::{Kernel, Win32Process};

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

    // parallel physical memory access
    {
        let pool = (0..8).map(|_| connector.clone()).collect::<Vec<_>>();

        let threads = pool.into_iter().map(|c| {
            thread::spawn(|| {
                Kernel::builder(c).build_default_caches().build().unwrap();
            })
        }).collect::<Vec<_>>();

        threads.into_iter().for_each(|t| t.join().unwrap());
    }

    // parallel virtual memory access
    {
        let mut kernel = Kernel::builder(&mut connector).build_default_caches().build().unwrap();

        // ... clone kernel?
        //let pool = (0..8).map(|_| connector.clone()).collect::<Vec<_>>();

        let pi = kernel.process_info("wininit.exe").unwrap();

        /*
        let p = Win32Process::with_kernel(kernel, proc_info)

        for c in pool.into_iter() {
            thread::spawn(move || {
                // ...
            });
        }
        */
    }

}
