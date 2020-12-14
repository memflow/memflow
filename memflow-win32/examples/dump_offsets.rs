use std::fs::File;
use std::io::Write;

use clap::*;
use log::{error, Level};

use memflow::connector::*;

use memflow_win32::prelude::{Kernel, Win32OffsetFile};

pub fn main() {
    let matches = App::new("dump offsets example")
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
        .arg(
            Arg::with_name("args")
                .long("args")
                .short("a")
                .takes_value(true)
                .default_value(""),
        )
        .arg(
            Arg::with_name("output")
                .long("output")
                .short("o")
                .takes_value(true),
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

    // create inventory + connector
    let inventory = unsafe { ConnectorInventory::scan() };
    let connector = unsafe {
        inventory.create_connector(
            matches.value_of("connector").unwrap(),
            &ConnectorArgs::parse(matches.value_of("args").unwrap()).unwrap(),
        )
    }
    .unwrap();

    let kernel = Kernel::builder(connector)
        .build_default_caches()
        .build()
        .unwrap();

    let winver = kernel.kernel_info.kernel_winver;

    if winver != (0, 0).into() {
        let offsets = if let Some(guid) = &kernel.kernel_info.kernel_guid {
            Win32OffsetFile {
                pdb_file_name: guid.file_name.as_str().into(),
                pdb_guid: guid.guid.as_str().into(),

                arch: kernel.kernel_info.start_block.arch.into(),

                nt_major_version: winver.major_version(),
                nt_minor_version: winver.minor_version(),
                nt_build_number: winver.build_number(),

                offsets: kernel.offsets.into(),
            }
        } else {
            Win32OffsetFile {
                pdb_file_name: Default::default(),
                pdb_guid: Default::default(),

                arch: kernel.kernel_info.start_block.arch.into(),

                nt_major_version: winver.major_version(),
                nt_minor_version: winver.minor_version(),
                nt_build_number: winver.build_number(),

                offsets: kernel.offsets.into(),
            }
        };

        // write offsets to file
        let offsetstr = toml::to_string_pretty(&offsets).unwrap();
        match matches.value_of("output") {
            Some(output) => {
                let mut file = File::create(output).unwrap();
                file.write_all(offsetstr.as_bytes()).unwrap();
            }
            None => println!("{}", offsetstr),
        }
    } else {
        error!("kernel version has to be valid in order to generate a offsets file");
    }
}
