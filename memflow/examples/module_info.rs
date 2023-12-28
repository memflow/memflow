/// A simple process list example using memflow
use clap::*;
use log::Level;

use memflow::prelude::v1::*;

fn main() -> Result<()> {
    let matches = parse_args();
    let (chain, proc_name, module_name) = extract_args(&matches)?;

    // create inventory + os
    let inventory = Inventory::scan();
    let os = inventory.builder().os_chain(chain).build()?;

    let mut process = os
        .into_process_by_name(proc_name)
        .expect("unable to find process");
    println!("found process: {:?}", process.info());

    let module = process
        .module_by_name(module_name)
        .expect("unable to retrieve module");

    println!(
        "{}:\nBase - {:x}\nSize - {:x}\nArch - {:?}\nFilename - {}",
        module.name, module.base, module.size, module.arch, module.path
    );

    if let Ok(s) = process
        .module_section_list(&module)
        .map_err(|e| panic!("{}", e))
    {
        println!("Sections:");

        for s in s {
            println!("0x{:0>8x} 0x{:0>8x} {}", s.base, s.size, s.name);
        }
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
            Arg::new("module")
                .long("module")
                .short('m')
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
        matches.get_one::<String>("module").unwrap(),
    ))
}
