/*!
A simple envar list and find example using memflow

# Usage:
```bash
cargo run --release --example envars_list -- -vvv -c memraw:/path/to/mem.raw --os win32 --process explorer.exe --envar USERDOMAIN
```
*/
use clap::*;
use log::Level;

use memflow::prelude::v1::*;

fn main() -> Result<()> {
    let matches = parse_args();
    let (chain, proc_name, envar) = extract_args(&matches)?;

    let mut inventory = Inventory::scan();
    let os = inventory.builder().os_chain(chain).build()?;

    let mut process = os
        .into_process_by_name(proc_name)
        .expect("unable to find process");
    println!("found process: {:?}", process.info());

    println!(
        "{:>11} {:>11} {:>11} {:>11} {:<}",
        "BASE", "SIZE", "MOD ARCH", "NAME", "PATH"
    );

    let envar_list = process
        .envar_list()
        .expect("unable to retrieve environment variables list");

    println!("   VARIABLE | VALUE");

    for ev in envar_list {
        println!("    {}={}", ev.name.as_ref(), ev.value.as_ref());
    }

    match process.envar_by_name(envar) {
        Ok(variable) => println!("FOUND {:?}", variable),
        Err(_) => println!("ENVAR NOT FOUND"),
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
            Arg::new("envar")
                .long("envar")
                .short('e')
                .action(ArgAction::Set)
                .required(true),
        )
        .get_matches()
}

fn extract_args(matches: &ArgMatches) -> Result<(OsChain<'_>, &str, &str)> {
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
        matches.get_one::<String>("envar").unwrap(),
    ))
}
