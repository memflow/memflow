use clap::{crate_authors, crate_version, App, Arg};
use log::Level;
/// A simple process list example using memflow
use memflow::prelude::v1::*;

fn main() -> Result<()> {
    let (conn_name, conn_args, os_name, os_args) = parse_args()?;

    // create connector + os
    let inventory = Inventory::scan();
    let mut os = {
        let builder = inventory.builder();

        if let Some(conn_name) = conn_name {
            builder
                .connector(&conn_name)
                .args(conn_args)
                .os(&os_name)
                .args(os_args)
                .build()
        } else {
            builder.os(&os_name).args(os_args).build()
        }
    }?;

    let process_list = os.process_info_list()?;

    // Print process list, formatted
    println!(
        "{:>5} {:>10} {:>10} {:<}",
        "PID", "SYS ARCH", "PROC ARCH", "NAME"
    );

    for p in process_list {
        println!(
            "{:>5} {:^10} {:^10} {} ({}) ({:?})",
            p.pid, p.sys_arch, p.proc_arch, p.name, p.command_line, p.state
        );
    }

    Ok(())
}

fn parse_args() -> Result<(Option<String>, Args, String, Args)> {
    let matches = App::new("mfps")
        .version(crate_version!())
        .author(crate_authors!())
        .arg(Arg::with_name("verbose").short("v").multiple(true))
        .arg(
            Arg::with_name("connector")
                .long("connector")
                .short("c")
                .takes_value(true)
                .required(false),
        )
        .arg(
            Arg::with_name("connector-args")
                .long("connector-args")
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
        matches.value_of("connector").map(ToString::to_string),
        Args::parse(matches.value_of("connector-args").ok_or_else(|| {
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
