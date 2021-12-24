use clap::{crate_authors, crate_version, App, Arg, ArgMatches};
use log::Level;
/// A simple process list example using memflow
use memflow::prelude::v1::*;

fn main() -> Result<()> {
    let matches = parse_args();
    let chain = extract_args(&matches)?;

    // create inventory + os
    let inventory = Inventory::scan();
    let mut os = inventory.builder().os_chain(chain).build()?;

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

fn parse_args() -> ArgMatches<'static> {
    App::new("mfps example")
        .version(crate_version!())
        .author(crate_authors!())
        .arg(Arg::with_name("verbose").short("v").multiple(true))
        .arg(
            Arg::with_name("connector")
                .long("connector")
                .short("c")
                .takes_value(true)
                .required(false)
                .multiple(true),
        )
        .arg(
            Arg::with_name("os")
                .long("os")
                .short("o")
                .takes_value(true)
                .required(true)
                .multiple(true),
        )
        .get_matches()
}

fn extract_args<'a>(matches: &'a ArgMatches) -> Result<OsChain<'a>> {
    // set log level
    let level = match matches.occurrences_of("verbose") {
        0 => Level::Error,
        1 => Level::Warn,
        2 => Level::Info,
        3 => Level::Debug,
        4 => Level::Trace,
        _ => Level::Trace,
    };

    simple_logger::SimpleLogger::new().init().unwrap();
    log::set_max_level(level.to_level_filter());

    if let Some(((conn_idx, conn), (os_idx, os))) = matches
        .indices_of("connector")
        .zip(matches.values_of("connector"))
        .zip(matches.indices_of("os").zip(matches.values_of("os")))
    {
        Ok(OsChain::new(conn_idx.zip(conn), os_idx.zip(os))?)
    } else {
        Err(ErrorKind::ArgValidation.into())
    }
}
