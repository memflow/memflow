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

fn parse_args() -> ArgMatches<'static> {
    App::new("keyboard example")
        .version(crate_version!())
        .author(crate_authors!())
        .arg(Arg::with_name("verbose").short("v").multiple(true))
        .arg(
            Arg::with_name("connector")
                .long("connector")
                .short("c")
                .takes_value(true)
                .required(false)
                .multiple(true),
        )
        .arg(
            Arg::with_name("os")
                .long("os")
                .short("o")
                .takes_value(true)
                .required(true)
                .multiple(true),
        )
        .get_matches()
}

fn extract_args<'a>(matches: &'a ArgMatches) -> Result<OsChain<'a>> {
    // set log level
    let level = match matches.occurrences_of("verbose") {
        0 => Level::Error,
        1 => Level::Warn,
        2 => Level::Info,
        3 => Level::Debug,
        4 => Level::Trace,
        _ => Level::Trace,
    };

    simple_logger::SimpleLogger::new().init().unwrap();
    log::set_max_level(level.to_level_filter());

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
