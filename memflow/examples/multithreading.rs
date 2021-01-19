use std::thread;

use clap::*;
use log::{info, Level};

use memflow::error::{Error, Result};
use memflow::os::*;
use memflow::plugins::*;

pub fn parallel_init(
    connector: ConnectorInstance,
    inventory: &Inventory,
    os_name: &str,
    os_args: &Args,
) {
    rayon::scope(|s| {
        (0..8)
            .map(|_| connector.clone())
            .into_iter()
            .map(|c| {
                s.spawn(move |_| {
                    inventory.create_os(os_name, c, os_args).unwrap();
                })
            })
            .count()
    });
}

pub fn parallel_kernels(kernel: KernelInstance) {
    (0..8)
        .map(|_| kernel.clone())
        .into_iter()
        .map(|mut k| {
            thread::spawn(move || {
                let _eprocesses = k.process_address_list().unwrap();
            })
        })
        .for_each(|t| t.join().unwrap());
}

pub fn parallel_processes(kernel: KernelInstance) {
    let process = kernel.into_process_by_name("wininit.exe").unwrap();

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
    let (conn_name, conn_args, os_name, os_args, log_level) = parse_args().unwrap();

    simple_logger::SimpleLogger::new()
        .with_level(log_level.to_level_filter())
        .init()
        .unwrap();

    // create inventory + connector
    let inventory = Inventory::scan();
    let connector = inventory
        .create_connector(&conn_name, None, &conn_args)
        .unwrap();

    // parallel test functions
    // see each function's implementation for further details

    parallel_init(connector.clone(), &inventory, &os_name, &os_args);

    let kernel = inventory.create_os(&os_name, connector, &os_args).unwrap();

    parallel_kernels(kernel.clone());

    parallel_processes(kernel);
}

fn parse_args() -> Result<(String, Args, String, Args, log::Level)> {
    let matches = App::new("multithreading example")
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
            Arg::with_name("conn-args")
                .long("conn-args")
                .short("x")
                .takes_value(true)
                .default_value(""),
        )
        .arg(
            Arg::with_name("os")
                .long("os")
                .short("o")
                .takes_value(true)
                .required(true),
        )
        .arg(
            Arg::with_name("os-args")
                .long("os-args")
                .short("y")
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

    Ok((
        matches
            .value_of("connector")
            .ok_or(Error::Other("failed to parse connector"))?
            .into(),
        Args::parse(
            matches
                .value_of("conn-args")
                .ok_or(Error::Other("failed to parse connector args"))?,
        )?,
        matches
            .value_of("os")
            .ok_or(Error::Other("failed to parse os"))?
            .into(),
        Args::parse(
            matches
                .value_of("os-args")
                .ok_or(Error::Other("failed to parse os args"))?,
        )?,
        level,
    ))
}
