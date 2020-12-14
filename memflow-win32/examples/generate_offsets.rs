use clap::*;
use log::{error, Level};
use std::fs::{create_dir_all, File};
use std::io::Write;
use std::path::PathBuf;

use memflow_win32::prelude::{
    SymbolStore, Win32GUID, Win32OffsetFile, Win32Offsets, Win32OffsetsArchitecture, Win32Version,
};

pub fn main() {
    let matches = App::new("generate offsets example")
        .version(crate_version!())
        .author(crate_authors!())
        .arg(Arg::with_name("verbose").short("v").multiple(true))
        .arg(
            Arg::with_name("output")
                .long("output")
                .short("o")
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

    let win_ids = vec![
        /*
        (
            Win32Version::new(5, 2, 3790),
            Win32GUID::new("ntkrnlmp.pdb", "82DCF67A38274C9CA99B60B421D2786D2"),
        ),
        */
        (
            Win32Version::new(6, 1, 7601),
            Win32OffsetsArchitecture::X86,
            Win32GUID::new("ntkrpamp.pdb", "684DA42A30CC450F81C535B4D18944B12"),
        ),
        (
            Win32Version::new(6, 1, 7601),
            Win32OffsetsArchitecture::X64,
            Win32GUID::new("ntkrnlmp.pdb", "ECE191A20CFF4465AE46DF96C22638451"),
        ),
        (
            Win32Version::new(10, 0, 18362),
            Win32OffsetsArchitecture::X64,
            Win32GUID::new("ntkrnlmp.pdb", "0AFB69F5FD264D54673570E37B38A3181"),
        ),
        (
            Win32Version::new(10, 0, 19041),
            Win32OffsetsArchitecture::X64,
            Win32GUID::new("ntkrnlmp.pdb", "BBED7C2955FBE4522AAA23F4B8677AD91"),
        ),
        (
            Win32Version::new(10, 0, 19041),
            Win32OffsetsArchitecture::X64,
            Win32GUID::new("ntkrnlmp.pdb", "1C9875F76C8F0FBF3EB9A9D7C1C274061"),
        ),
        (
            Win32Version::new(10, 0, 19041),
            Win32OffsetsArchitecture::X86,
            Win32GUID::new("ntkrpamp.pdb", "1B1D6AA205E1C87DC63A314ACAA50B491"),
        ),
    ];

    let out_dir = matches.value_of("output").unwrap();
    create_dir_all(out_dir).unwrap();

    for win_id in win_ids.into_iter() {
        if let Ok(offsets) = Win32Offsets::builder()
            .symbol_store(SymbolStore::new())
            .guid(win_id.2.clone())
            .build()
        {
            let offset_file = Win32OffsetFile {
                pdb_file_name: win_id.2.file_name.as_str().into(),
                pdb_guid: win_id.2.guid.as_str().into(),

                nt_major_version: win_id.0.major_version(),
                nt_minor_version: win_id.0.minor_version(),
                nt_build_number: win_id.0.build_number(),

                arch: win_id.1,

                offsets: offsets.0,
            };

            let offsetstr = toml::to_string_pretty(&offset_file).unwrap();

            let file_name = format!(
                "{}_{}_{}_{}_{}.toml",
                win_id.0.major_version(),
                win_id.0.minor_version(),
                win_id.0.build_number(),
                win_id.1.to_string(),
                win_id.2.guid,
            );

            let mut file =
                File::create([out_dir, &file_name].iter().collect::<PathBuf>().as_path()).unwrap();
            file.write_all(offsetstr.as_bytes()).unwrap();
        } else {
            error!(
                "unable to find offsets for {} {:?} {:?}",
                win_id.0, win_id.1, win_id.2
            )
        }
    }
}
