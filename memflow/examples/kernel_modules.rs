use clap::{crate_authors, crate_version, App, Arg, ArgMatches};
use log::Level;
/// A simple kernel module list example using memflow
use memflow::prelude::v1::*;

fn main() -> Result<()> {
    let matches = parse_args();
    let chain = extract_args(&matches)?;

    // create inventory + os
    let inventory = Inventory::scan();
    let mut os = inventory.builder().os_chain(chain).build()?;

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

fn parse_args() -> ArgMatches {
    App::new("kernel_modules example")
        .version(crate_version!())
        .author(crate_authors!())
        .arg(Arg::new("verbose").short('v').multiple_occurrences(true))
        .arg(
            Arg::new("connector")
                .long("connector")
                .short('c')
                .takes_value(true)
                .required(false)
                .multiple_values(true),
        )
        .arg(
            Arg::new("os")
                .long("os")
                .short('o')
                .takes_value(true)
                .required(true)
                .multiple_values(true),
        )
        .get_matches()
}

fn extract_args<'a>(matches: &'a ArgMatches) -> Result<OsChain<'a>> {
    let log_level = match matches.occurrences_of("verbose") {
        0 => Level::Error,
        1 => Level::Warn,
        2 => Level::Info,
        3 => Level::Debug,
        4 => Level::Trace,
        _ => Level::Trace,
    };
    simplelog::TermLogger::init(
        log_level.to_level_filter(),
        simplelog::Config::default(),
        simplelog::TerminalMode::Stdout,
        simplelog::ColorChoice::Auto,
    )
    .unwrap();

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
