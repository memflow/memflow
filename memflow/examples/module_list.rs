use clap::{crate_authors, crate_version, App, Arg, ArgMatches};
use log::Level;
/// A simple process list example using memflow
use memflow::prelude::v1::*;

fn main() -> Result<()> {
    let matches = parse_args();
    let (chain, proc_name) = extract_args(&matches)?;

    // create inventory + os
    let inventory = Inventory::scan();
    let os = inventory.builder().os_chain(chain).build()?;

    let mut process = os
        .into_process_by_name(&proc_name)
        .expect("unable to find process");
    println!("found process: {:?}", process.info());

    let module_list = process
        .module_list()
        .expect("unable to retrieve module list");

    // Print module list, formatted
    println!(
        "{:>11} {:>11} {:>11} {:>11} {:<}",
        "BASE", "SIZE", "MOD ARCH", "NAME", "PATH"
    );

    for m in module_list {
        println!(
            "0x{:x>8} 0x{:x>8} {:^10} {} ({})",
            m.base, m.size, m.arch, m.name, m.path
        );
    }

    Ok(())
}

fn parse_args() -> ArgMatches<'static> {
    App::new("module_list example")
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
        .arg(
            Arg::with_name("process")
                .long("process")
                .short("p")
                .takes_value(true)
                .required(true),
        )
        .get_matches()
}

fn extract_args<'a>(matches: &'a ArgMatches) -> Result<(OsChain<'a>, &'a str)> {
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
        Ok((
            OsChain::new(conn_idx.zip(conn), os_idx.zip(os))?,
            matches.value_of("process").unwrap(),
        ))
    } else {
        Err(ErrorKind::ArgValidation.into())
    }
}
