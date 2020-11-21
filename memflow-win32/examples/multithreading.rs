use std::thread;

use clap::*;
use log::{info, Level};

use memflow::connector::*;
use memflow::mem::*;

use memflow_win32::win32::Kernel;

pub fn parallel_init<T: PhysicalMemory + Clone + 'static>(connector: T) {
    (0..8)
        .map(|_| connector.clone())
        .into_iter()
        .map(|c| {
            thread::spawn(move || {
                Kernel::builder(c)
                    .no_symbol_store()
                    .build_default_caches()
                    .build()
                    .unwrap();
            })
        })
        .for_each(|t| t.join().unwrap());
}

pub fn parallel_kernels<T: PhysicalMemory + Clone + 'static>(connector: T) {
    let kernel = Kernel::builder(connector).build().unwrap();

    (0..8)
        .map(|_| kernel.clone())
        .into_iter()
        .map(|mut k| {
            thread::spawn(move || {
                let _eprocesses = k.eprocess_list().unwrap();
            })
        })
        .for_each(|t| t.join().unwrap());
}

pub fn parallel_kernels_cached<T: PhysicalMemory + Clone + 'static>(connector: T) {
    let kernel = Kernel::builder(connector)
        .build_default_caches()
        .build()
        .unwrap();

    (0..8)
        .map(|_| kernel.clone())
        .into_iter()
        .map(|mut k| {
            thread::spawn(move || {
                let eprocesses = k.eprocess_list().unwrap();
                info!("eprocesses list fetched: {}", eprocesses.len());
            })
        })
        .for_each(|t| t.join().unwrap());
}

pub fn parallel_processes<T: PhysicalMemory + Clone + 'static>(connector: T) {
    let kernel = Kernel::builder(connector)
        .build_default_caches()
        .build()
        .unwrap();

    let process = kernel.into_process("wininit.exe").unwrap();

    (0..8)
        .map(|_| process.clone())
        .into_iter()
        .map(|mut p| {
            thread::spawn(move || {
                let module_list = p.module_list().unwrap();
                info!("wininit.exe module_list: {}", module_list.len());
            })
        })
        .for_each(|t| t.join().unwrap());
}

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

    // parallel test functions
    // see each function's implementation for further details

    parallel_init(connector.clone());

    parallel_kernels(connector.clone());

    parallel_kernels_cached(connector.clone());

    parallel_processes(connector);
}
