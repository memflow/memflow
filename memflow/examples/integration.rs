use memflow::os::*;
use memflow::plugins::*;

use clap::*;
use log::Level;

use memflow::error::{Error, Result};

use colored::*;

static mut HAD_ERROR: bool = false;

fn main() -> Result<()> {
    let (connector, args_str, os_name, os_args_str, sysproc, kernel_mods) = parse_args();

    let args = Args::parse(&args_str)?;
    let os_args = Args::parse(&os_args_str)?;

    // create inventory + connector
    let inventory = Inventory::scan();
    let connector = inventory.create_connector(&connector, None, &args)?;

    let kernel_result = inventory.create_os(&os_name, Some(connector), &os_args);
    println!("Kernel::build ... {}", ok_str(&kernel_result));
    let mut kernel = kernel_result?;

    {
        println!("Kernel info:");
        let base_info = kernel.info();
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
        if let Ok(modules) = kernel_modules(&mut kernel) {
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
        let prc_list = kernel.process_info_list()?;
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
            Err(Error::Other(
                "Some errors encountered, not all functionality may be present!",
            ))
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

fn kernel_modules(kernel: &mut impl OS) -> Result<Vec<ModuleInfo>> {
    let modules = kernel.module_list().map_err(From::from);
    println!("kernel modules ... {}", ok_str(&modules));
    modules
}

fn parse_args() -> (String, String, String, String, String, String) {
    let matches = App::new("multithreading example")
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
            Arg::with_name("conn-args")
                .long("conn-args")
                .short("x")
                .takes_value(true)
                .default_value(""),
        )
        .arg(
            Arg::with_name("os")
                .long("os")
                .short("o")
                .takes_value(true)
                .required(true),
        )
        .arg(
            Arg::with_name("os-args")
                .long("os-args")
                .short("y")
                .takes_value(true)
                .default_value(""),
        )
        .arg(
            Arg::with_name("system-proc")
                .long("system-proc")
                .short("p")
                .takes_value(true)
                .default_value("lsass.exe"),
        )
        .arg(
            Arg::with_name("kernel-mods")
                .long("kernel-mods")
                .short("k")
                .takes_value(true)
                .default_value("ntoskrnl.exe,hal.dll"),
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

    (
        matches.value_of("connector").unwrap().into(),
        matches.value_of("conn-args").unwrap().into(),
        matches.value_of("os").unwrap().into(),
        matches.value_of("os-args").unwrap().into(),
        matches.value_of("system-proc").unwrap().into(),
        matches.value_of("kernel-mods").unwrap().into(),
    )
}
