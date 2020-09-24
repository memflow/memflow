use std::thread;

use clap::*;
use log::{info, Level};

use memflow::connector::*;
use memflow::mem::*;

use memflow_win32::win32::Kernel;

pub fn parallel_init<T: PhysicalMemory + Clone + 'static>(connector: T) {
    let pool = (0..8).map(|_| connector.clone()).collect::<Vec<_>>();

    let threads = pool
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
        .collect::<Vec<_>>();

    threads.into_iter().for_each(|t| t.join().unwrap());
}

pub fn parallel_kernels<T: PhysicalMemory + Clone + 'static>(connector: T) {
    let kernel = Kernel::builder(connector).build().unwrap();

    let pool = (0..8).map(|_| kernel.clone()).collect::<Vec<_>>();

    let threads = pool
        .into_iter()
        .map(|mut k| {
            thread::spawn(move || {
                let _eprocesses = k.eprocess_list().unwrap();
            })
        })
        .collect::<Vec<_>>();

    threads.into_iter().for_each(|t| t.join().unwrap());
}

pub fn parallel_kernels_cached<T: PhysicalMemory + Clone + 'static>(connector: T) {
    let kernel = Kernel::builder(connector)
        .build_default_caches()
        .build()
        .unwrap();

    let pool = (0..8).map(|_| kernel.clone()).collect::<Vec<_>>();

    let threads = pool
        .into_iter()
        .map(|mut k| {
            thread::spawn(move || {
                let eprocesses = k.eprocess_list().unwrap();
                info!("eprocesses list fetched: {}", eprocesses.len());
            })
        })
        .collect::<Vec<_>>();

    threads.into_iter().for_each(|t| t.join().unwrap());
}

pub fn parallel_processes<T: PhysicalMemory + Clone + 'static>(connector: T) {
    let kernel = Kernel::builder(connector)
        .build_default_caches()
        .build()
        .unwrap();

    let process = kernel.into_process("wininit.exe").unwrap();

    let pool = (0..8).map(|_| process.clone()).collect::<Vec<_>>();

    let threads = pool
        .into_iter()
        .map(|mut p| {
            thread::spawn(move || {
                let module_list = p.module_list().unwrap();
                info!("wininit.exe module_list: {}", module_list.len());
            })
        })
        .collect::<Vec<_>>();

    threads.into_iter().for_each(|t| t.join().unwrap());
}

#[cfg(not(windows))]
fn elevate_privileges() {
    sudo::escalate_if_needed().expect("failed to elevate privileges");
}

#[cfg(windows)]
fn elevate_privileges() {
    error!("elevate privileges is not available on windows");
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
        .arg(
            Arg::with_name("elevate")
                .short("E")
                .long("elevate")
                .help("elevate privileges upon start")
                .takes_value(false)
                .required(false),
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

    if matches.is_present("elevate") {
        elevate_privileges();
    }

    // create inventory + connector
    let inventory = unsafe { ConnectorInventory::try_new() }.unwrap();
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
