/// A simple process list example using memflow
use clap::*;
use log::Level;

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

fn parse_args() -> ArgMatches {
    Command::new("mfps example")
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
        .get_matches()
}

fn extract_args(matches: &ArgMatches) -> Result<OsChain<'_>> {
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

    OsChain::new(conn_iter, os_iter)
}
