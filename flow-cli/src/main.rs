mod init;
use init::*;

mod cli;
use cli::*;

#[macro_use]
extern crate clap;
use clap::App;

use log::{trace, Level};
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

    let mut phys_mem = match argv.value_of("connector").unwrap_or_else(|| "bridge") {
        /* "bridge" => {
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
        } */
        "qemu_procfs" => init_qemu_procfs::init_qemu_procfs(&argv)?,
        _ => return Err(Error::new("the connector requested does not exist")),
    };

    let kernel_info = KernelInfo::find(&mut phys_mem)?;

    let phys_page_cache = PageCache::new(
        kernel_info.start_block.arch,
        Length::from_mb(32),
        PageType::PAGE_TABLE | PageType::READ_ONLY,
        TimedCacheValidator::new(Duration::from_millis(1000).into()),
    );
    let mem_cached = CachedMemoryAccess::with(&mut phys_mem, phys_page_cache);

    let tlb_cache = TLBCache::new(
        2048.into(),
        TimedCacheValidator::new(Duration::from_millis(1000).into()),
    );
    let mut vat = TranslateArch::new(kernel_info.start_block.arch);
    let vat_cached =
        CachedVirtualTranslate::with(&mut vat, tlb_cache, kernel_info.start_block.arch);

    let offsets = Win32Offsets::try_with_guid(&kernel_info.kernel_guid)?;
    trace!("offsets: {:?}", offsets);
    let mut kernel = Kernel::new(mem_cached, vat_cached, offsets, kernel_info);

    let mut win32 = Win32Interface::new(&mut kernel)?;
    win32.run()
}
