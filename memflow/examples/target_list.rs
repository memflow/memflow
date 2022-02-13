use memflow::plugins::*;

use clap::*;
use log::Level;

fn main() {
    let connector = parse_args();

    // create inventory
    let inventory = Inventory::scan();

    // try to get help text
    println!(
        "Connector help:\n{}",
        inventory.connector_help(&connector).unwrap_or_default()
    );

    // try to get target list
    let targets = inventory
        .connector_target_list(&connector)
        .expect("unable to get target list");

    println!("Targets for connector `{}`:", &connector);
    targets.iter().for_each(|t| println!("- {}", t.name));
}

fn parse_args() -> String {
    let matches = App::new("multithreading example")
        .version(crate_version!())
        .author(crate_authors!())
        .arg(Arg::new("verbose").short('v').multiple_occurrences(true))
        .arg(
            Arg::new("connector")
                .long("connector")
                .short('c')
                .takes_value(true)
                .required(true),
        )
        .get_matches();

    // set log level
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

    matches.value_of("connector").unwrap().into()
}
