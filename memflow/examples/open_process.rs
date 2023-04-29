/*!
This example shows how to use a dynamically loaded connector in conjunction
with a dynamically loaded os plugin. This example uses the `Inventory` feature of memflow
to load all the required plugins. Plugins are also chainable by providing the `--os`
and `--connector` arguments multiple times.

The example showcases how to retrieve extended process info data,
opening the process and getting a list of all modules.

Additionally the process can be initialized with a (optional) user-provided DTB (Directory Table Base).

# Usage:
Open process and load the given module with the default dtb.
```bash
cargo run --release --example open_process -- -vvv -c kvm --os win32 --process explorer.exe -m KERNEL32.DLL
```

Overwrite dtb with a custom one:
```bash
cargo run --release --example open_process -- -vvv -c kvm --os win32 --process explorer.exe -m KERNEL32.DLL --dtb DEADBEEF
```
*/
use clap::*;
use log::Level;

use memflow::prelude::v1::*;

fn main() -> Result<()> {
    let matches = parse_args();
    let (chain, proc_name, module_name, dtb) = extract_args(&matches)?;

    // create inventory + os
    let inventory = Inventory::scan();
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

    // retrieve module info
    let module_info = process
        .module_by_name(module_name)
        .expect("unable to find module in process");
    println!("{module_info:?}");

    // count exports
    let export_count = process
        .module_export_list(&module_info)
        .expect("unable to get exports")
        .len();
    println!("Exports: {export_count}");

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
            Arg::new("module")
                .long("module")
                .short('m')
                .action(ArgAction::Set)
                .required(true)
                .default_value("KERNEL32.DLL"),
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

fn extract_args(matches: &ArgMatches) -> Result<(OsChain<'_>, &str, &str, Option<Address>)> {
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
        matches.get_one::<String>("module").unwrap(),
        matches
            .get_one::<String>("dtb")
            .map(|dtb| umem::from_str_radix(dtb, 16).expect("unable to parse dtb as a hex number"))
            .map(Address::from),
    ))
}
