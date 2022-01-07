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
        .into_process_by_name(proc_name)
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
            "0x{:0>8x} 0x{:0>8x} {:^10} {} ({})",
            m.base, m.size, m.arch, m.name, m.path
        );
    }

    Ok(())
}

fn parse_args() -> ArgMatches {
    App::new("module_list example")
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
        .arg(
            Arg::new("process")
                .long("process")
                .short('p')
                .takes_value(true)
                .required(true),
        )
        .get_matches()
}

fn extract_args<'a>(matches: &'a ArgMatches) -> Result<(OsChain<'a>, &'a str)> {
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

    let conn_iter = matches
        .indices_of("connector")
        .zip(matches.values_of("connector"))
        .map(|(a, b)| a.zip(b))
        .into_iter()
        .flatten();

    let os_iter = matches
        .indices_of("os")
        .zip(matches.values_of("os"))
        .map(|(a, b)| a.zip(b))
        .into_iter()
        .flatten();

    Ok((
        OsChain::new(conn_iter, os_iter)?,
        matches.value_of("process").unwrap(),
    ))
}
