use clap::*;
use log::Level;

use memflow::prelude::v1::*;

#[derive(Debug, Default, Batcher)]
struct DOSHeader {
    #[batch(offset = 0x0)]
    e_magic: u16,
    #[batch(offset = 0x2)]
    e_cblp: u16,
    #[batch(offset = 0x18)]
    e_lfarlc: u16,
}

// cargo run --example batcher -- -vv --os="native" --process="notepad.exe"
fn main() -> Result<()> {
    let matches = parse_args();
    let (chain, proc_name, dtb) = extract_args(&matches)?;

    // create inventory + os
    let mut inventory = Inventory::scan();
    let mut os = inventory.builder().os_chain(chain).build()?;

    let mut process = if let Some(dtb) = dtb {
        // open process with a custom dtb
        let mut proc_info = os
            .process_info_by_name(proc_name)
            .expect("unable to find process");
        proc_info.dtb1 = dtb;
        os.into_process_by_info(proc_info)
            .expect("unable to open process")
    } else {
        // use default dtb
        os.into_process_by_name(proc_name)
            .expect("unable to find process")
    };
    println!("{:?}", process.info());

    // Alternatively the dtb can be modified after the process has been initialized:
    if let Some(dtb) = dtb {
        process
            .set_dtb(dtb, Address::invalid())
            .expect("unable to modify process dtb");
    }

    let module = process
        .module_by_name(proc_name)
        .expect("unable to find module");

    // batch read dos header infos
    let pe_base = module.base;
    let mut dos_header = DOSHeader::default();
    dos_header.read_all_batched(process, pe_base);

    println!("partial dos header data: {:x?}", dos_header);

    Ok(())
}

fn parse_args() -> ArgMatches {
    Command::new("open_process example")
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
                .required(true)
                .default_value("explorer.exe"),
        )
        .arg(
            Arg::new("dtb")
                .long("dtb")
                .short('d')
                .action(ArgAction::Set)
                .required(false),
        )
        .get_matches()
}

fn extract_args(matches: &ArgMatches) -> Result<(OsChain<'_>, &str, Option<Address>)> {
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
        matches
            .get_one::<String>("dtb")
            .map(|dtb| umem::from_str_radix(dtb, 16).expect("unable to parse dtb as a hex number"))
            .map(Address::from),
    ))
}
