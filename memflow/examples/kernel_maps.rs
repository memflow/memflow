/// A simple kernel module list example using memflow
use clap::{crate_authors, crate_version, Arg, ArgMatches, Command};
use log::Level;
use memflow::prelude::v1::*;

fn main() -> Result<()> {
    let matches = parse_args();
    let chain = extract_args(&matches)?;

    // create inventory + os
    let inventory = Inventory::scan();
    let mut os = inventory.builder().os_chain(chain).build()?;

    let vt = os
        .as_mut_impl_virtualtranslate()
        .expect("VirtualTranslate is not implemented for this OS plugin");

    // Print map list, formatted
    println!("{:>16} {:>12} {:<}", "ADDR", "SIZE", "TYPE");

    let callback = &mut |CTup3(addr, size, pagety)| {
        println!("{:>16x} {:>12x} {:<?}", addr, size, pagety);
        true
    };
    vt.virt_page_map(0, callback.into());

    Ok(())
}

fn parse_args() -> ArgMatches {
    Command::new("kernel_maps example")
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

fn extract_args(matches: &ArgMatches) -> Result<OsChain<'_>> {
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

    Ok(OsChain::new(conn_iter, os_iter)?)
}
