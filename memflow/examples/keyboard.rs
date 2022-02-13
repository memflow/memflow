use clap::{crate_authors, crate_version, App, Arg, ArgMatches};
use log::Level;
/// A simple keyboard example using memflow
use memflow::prelude::v1::*;

fn main() -> Result<()> {
    let matches = parse_args();
    let chain = extract_args(&matches)?;

    // create inventory + os
    let inventory = Inventory::scan();
    let os = inventory.builder().os_chain(chain).build()?;

    if !os.check_impl_oskeyboardinner() {
        return Err(
            Error(ErrorOrigin::Other, ErrorKind::UnsupportedOptionalFeature)
                .log_error("keyboard feature is not implemented for the given os plugin"),
        );
    }

    let mut keyboard = into!(os impl OsKeyboardInner).unwrap().into_keyboard()?;

    loop {
        println!("space down: {:?}", keyboard.is_down(0x20)); // VK_SPACE
        std::thread::sleep(std::time::Duration::from_millis(1000));
    }
}

fn parse_args() -> ArgMatches {
    App::new("keyboard example")
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

    OsChain::new(conn_iter, os_iter)
}
