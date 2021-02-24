use clap::{crate_authors, crate_version, App, Arg};
use log::Level;
/// A simple kernel module list example using memflow
use memflow::prelude::v1::*;

fn main() -> Result<()> {
    let (connector, conn_args, os, os_args) = parse_args()?;

    // create connector + os
    let mut os = Inventory::build_conn_os_combo(&connector, &conn_args, &os, &os_args)?;

    let module_list = os.module_list()?;

    // Print process list, formatted
    println!(
        "{:>16} {:>16} {:>8} {:>24} {:<}",
        "INTERNAL ADDR", "BASE", "SIZE", "NAME", "PATH"
    );

    for m in module_list {
        println!(
            "{:>16x} {:>16x} {:>8x} {:>24} {}",
            m.address, m.base, m.size, m.name, m.path
        );
    }

    Ok(())
}

fn parse_args() -> Result<(String, Args, String, Args)> {
    let matches = App::new("mfps")
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

    // initialize loggong
    simple_logger::SimpleLogger::new()
        .with_level(level.to_level_filter())
        .init()
        .unwrap();

    Ok((
        matches
            .value_of("connector")
            .ok_or_else(|| {
                Error(ErrorOrigin::Other, ErrorKind::Configuration)
                    .log_error("failed to parse connector")
            })?
            .into(),
        Args::parse(matches.value_of("conn-args").ok_or_else(|| {
            Error(ErrorOrigin::Other, ErrorKind::Configuration)
                .log_error("failed to parse connector args")
        })?)?,
        matches
            .value_of("os")
            .ok_or_else(|| {
                Error(ErrorOrigin::Other, ErrorKind::Configuration).log_error("failed to parse os")
            })?
            .into(),
        Args::parse(matches.value_of("os-args").ok_or_else(|| {
            Error(ErrorOrigin::Other, ErrorKind::Configuration).log_error("failed to parse os args")
        })?)?,
    ))
}
