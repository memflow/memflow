/// A simple kernel structuring example using memflow
use clap::*;
use log::Level;

use memflow::prelude::v1::*;
use offsetter::offset_debug;

//These offsets are for Windows 11 24H2, This is just an example of destructuring
//https://www.vergiliusproject.com/kernels/x64/windows-11/24h2/
offset_debug!(
    struct _EPROCESS  {
        //first bytes are the offset in the structure
        //useful as we dont need to convert the entire struct or pad any bytes, the macro will handle this
        0x0 pcb: _KPROCESS,
        0x2e0 peb: Pointer64<_PEB>,
    }
);

offset_debug!(
    struct _KPROCESS  {
        0x28 directory_table_base: Address,
    }
);

offset_debug!(
    struct _PEB {
        0x2 being_debugged: bool,
    }
);
//implement for Plain Old Data, this transmutes the read to the structure.
unsafe impl Pod for _EPROCESS {}
unsafe impl Pod for _PEB {}

fn main() -> Result<()> {
    let matches = parse_args();
    let chain = extract_args(&matches)?;

    // create inventory + os
    let mut inventory = Inventory::scan();
    let mut os = inventory.builder().os_chain(chain).build()?;

    //check if ntoskrnl is present here
    if os.primary_module()?.name != "ntoskrnl.exe".into() {
        panic!("Unsupported OS!")
    }

    let process_list = os.process_info_list()?;

    for p in process_list {
        let mut process = os.process_by_info(p.clone())?;

        let eprocess = process.read::<_EPROCESS>(p.address)?;
        let dtb = eprocess.pcb.directory_table_base;
        let pcb = eprocess.peb.read(&mut process)?;

        //internally memflow parses this from pdb. so its a good cross check
        if p.dtb1 != dtb {
            panic!("Invalid directory table base offset vs memflow expected value. Invalid offset or DTB invalidated");
        }
        // check if the process is being debugged, we can simply attach windbg to a given process
        if pcb.being_debugged {
            println!(
                "Process: {} {:>16} is_being_debugged: {}",
                p.name, dtb, pcb.being_debugged
            );
        }
    }

    Ok(())
}

fn parse_args() -> ArgMatches {
    Command::new("kernel_modules example")
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
        .get_matches()
}

fn extract_args(matches: &ArgMatches) -> Result<OsChain<'_>> {
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

    OsChain::new(conn_iter, os_iter)
}
