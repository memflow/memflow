use memflow::prelude::v1::*;
use memflow::prelude::v1::{ErrorKind, Result};

use clap::*;
use log::Level;

use colored::*;

static mut HAD_ERROR: bool = false;

fn main() -> Result<()> {
    let matches = parse_args();
    let (chain, sysproc, kernel_mods) = extract_args(&matches)?;

    // create inventory + os
    let inventory = Inventory::scan();
    let mut os = inventory.builder().os_chain(chain).build()?;

    {
        println!("Kernel info:");
        let base_info = os.info();
        println!(
            "base: {:x} ... {}",
            base_info.base,
            some_str(&base_info.base.non_null())
        );
        println!(
            "size: {:x} ... {}",
            base_info.size,
            bool_str(base_info.size != 0)
        );
        println!();
    }

    {
        let os_base = os.info().base;

        let mut out = [0u8; 32];
        let phys_mem = as_mut!(os impl PhysicalMemory).expect("no phys mem found");
        phys_mem.phys_read_into(0x1000.into(), &mut out).unwrap();
        println!("Kernel Physical Read: {out:?}");

        let virt_mem = as_mut!(os impl MemoryView).expect("no virt mem found");
        virt_mem.read_into(os_base, &mut out).unwrap();
        println!("Kernel Virtual Read: {out:?}");
    }

    {
        if let Ok(modules) = kernel_modules(&mut os) {
            for k in kernel_mods.split(',') {
                println!(
                    "{} ... {}",
                    k,
                    some_str(&modules.iter().find(|e| e.name.to_lowercase() == k))
                );
            }
        }
        println!();
    }

    {
        println!("Process List:");
        let prc_list = os.process_info_list()?;
        let lsass = prc_list
            .iter()
            .find(|p| p.name.to_string().to_lowercase() == sysproc);
        println!("{} ... {}", &sysproc, some_str(&lsass));
        println!();

        if let Some(prc) = lsass {
            println!("{} info:", prc.name);
            println!("pid: {} ... {}", prc.pid, bool_str(prc.pid < 10000));
        }
    }

    unsafe {
        if HAD_ERROR {
            Err(Error(ErrorOrigin::Other, ErrorKind::Unknown)
                .log_error("Some errors encountered, not all functionality may be present!"))
        } else {
            Ok(())
        }
    }
}

fn some_str<T>(r: &Option<T>) -> ColoredString {
    bool_str(r.is_some())
}

fn ok_str<T>(r: &Result<T>) -> ColoredString {
    bool_str(r.is_ok())
}

fn bool_str(b: bool) -> ColoredString {
    if b {
        "ok".green()
    } else {
        unsafe { HAD_ERROR = true };
        "error".red()
    }
}

fn kernel_modules(kernel: &mut impl Os) -> Result<Vec<ModuleInfo>> {
    let modules = kernel.module_list().map_err(From::from);
    println!("kernel modules ... {}", ok_str(&modules));
    modules
}

fn parse_args() -> ArgMatches {
    Command::new("integration example")
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
            Arg::new("system-proc")
                .long("system-proc")
                .short('p')
                .action(ArgAction::Set)
                .default_value("lsass.exe"),
        )
        .arg(
            Arg::new("kernel-mods")
                .long("kernel-mods")
                .short('k')
                .action(ArgAction::Set)
                .default_value("ntoskrnl.exe,hal.dll"),
        )
        .get_matches()
}

fn extract_args(matches: &ArgMatches) -> Result<(OsChain<'_>, &str, &str)> {
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
        matches.get_one::<String>("system-proc").unwrap(),
        matches.get_one::<String>("kernel-mods").unwrap(),
    ))
}
