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
    targets.iter().for_each(|t| println!("{:?}", t));
}

fn parse_args() -> String {
    let matches = App::new("multithreading example")
        .version(crate_version!())
        .author(crate_authors!())
        .arg(Arg::with_name("verbose").short("v").multiple(true))
        .arg(
            Arg::with_name("connector")
                .long("connector")
                .short("c")
                .takes_value(true)
                .required(true),
        )
        .get_matches();

    // set log level
    let level = match matches.occurrences_of("verbose") {
        0 => Level::Error,
        1 => Level::Warn,
        2 => Level::Info,
        3 => Level::Debug,
        4 => Level::Trace,
        _ => Level::Trace,
    };
    simple_logger::SimpleLogger::new()
        .with_level(level.to_level_filter())
        .init()
        .unwrap();

    matches.value_of("connector").unwrap().into()
}
