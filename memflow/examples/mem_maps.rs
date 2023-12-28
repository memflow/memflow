/// A simple process list example using memflow
use clap::*;
use log::Level;

use memflow::prelude::v1::*;

fn main() -> Result<()> {
    let matches = parse_args();
    let (chain, proc_name, gap_size) = extract_args(&matches)?;

    // create inventory + os
    let inventory = Inventory::scan();
    let os = inventory.builder().os_chain(chain).build()?;

    let mut process = os
        .into_process_by_name(proc_name)
        .expect("unable to find process");
    println!("found process: {:?}", process.info());

    let maps = process.mapped_mem_vec(gap_size);

    // Print module list, formatted
    println!("{:>11} {:>11} {:<}", "BASE", "SIZE", "FLAGS");

    for CTup3(a, s, p) in maps {
        println!("0x{a:0>8x} 0x{s:0>8x} {p:?}",);
    }

    Ok(())
}

fn parse_args() -> ArgMatches {
    Command::new("module_list example")
        .version(crate_version!())
        .author(crate_authors!())
        .arg(Arg::new("verbose").short('v').action(ArgAction::Count))
        .arg(
            Arg::new("connector")
                .long("connector")
                .short('c')
                .action(ArgAction::Append)
                .required(false),
        )
        .arg(
            Arg::new("os")
                .long("os")
                .short('o')
                .action(ArgAction::Append)
                .required(true),
        )
        .arg(
            Arg::new("process")
                .long("process")
                .short('p')
                .action(ArgAction::Set)
                .required(true),
        )
        .arg(
            Arg::new("gap-size")
                .long("gap-size")
                .short('g')
                .action(ArgAction::Set)
                .required(false),
        )
        .get_matches()
}

fn extract_args(matches: &ArgMatches) -> Result<(OsChain<'_>, &str, imem)> {
    let log_level = match matches.get_count("verbose") {
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
        .zip(matches.get_many::<String>("connector"))
        .map(|(a, b)| a.zip(b.map(String::as_str)))
        .into_iter()
        .flatten();

    let os_iter = matches
        .indices_of("os")
        .zip(matches.get_many::<String>("os"))
        .map(|(a, b)| a.zip(b.map(String::as_str)))
        .into_iter()
        .flatten();

    Ok((
        OsChain::new(conn_iter, os_iter)?,
        matches.get_one::<String>("process").unwrap(),
        matches
            .get_one::<&str>("gap-size")
            .unwrap_or(&"-1")
            .parse()
            .unwrap(),
    ))
}
