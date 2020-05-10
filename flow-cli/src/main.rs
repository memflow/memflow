mod init;
use init::*;

mod cli;
use cli::*;

#[macro_use]
extern crate clap;
use clap::App;

use log::Level;
use std::time::Duration;

use flow_core::timed_validator::*;
use flow_core::*;
use flow_core::{Error, Result};

use flow_win32::*;

fn main() -> Result<()> {
    let yaml = load_yaml!("cli.yml");
    let argv = App::from(yaml).get_matches();

    match argv.occurrences_of("verbose") {
        1 => simple_logger::init_with_level(Level::Warn).unwrap(),
        2 => simple_logger::init_with_level(Level::Info).unwrap(),
        3 => simple_logger::init_with_level(Level::Debug).unwrap(),
        4 => simple_logger::init_with_level(Level::Trace).unwrap(),
        _ => simple_logger::init_with_level(Level::Error).unwrap(),
    }

    match argv.value_of("connector").unwrap_or_else(|| "bridge") {
        "bridge" => {
            let mut conn = init_bridge::init_bridge(&argv).unwrap();
            let os = Win32::try_with(&mut conn)?;

            let cache = PageCache::new(
                os.start_block.arch,
                Length::from_mb(32),
                PageType::PAGE_TABLE | PageType::READ_ONLY,
                TimedCacheValidator::new(Duration::from_millis(1000).into()),
            );
            let mut mem = CachedMemoryAccess::with(&mut conn, cache);

            let mut win32 = Win32Interface::with(&mut mem, os)?;
            win32.run()
        }
        "qemu_procfs" => {
            let mut conn = init_qemu_procfs::init_qemu_procfs().unwrap();
            let os = Win32::try_with(&mut conn)?;

            let cache = PageCache::new(
                os.start_block.arch,
                Length::from_mb(32),
                PageType::PAGE_TABLE | PageType::READ_ONLY,
                TimedCacheValidator::new(Duration::from_millis(1000).into()),
            );
            let mut mem = CachedMemoryAccess::with(&mut conn, cache);

            let mut win32 = Win32Interface::with(&mut mem, os)?;
            win32.run()
        }
        _ => Err(Error::new("the connector requested does not exist")),
    }
}
