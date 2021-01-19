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

    let mut kernel = build_kernel(connector, &inventory, &os_name, &os_args)?;

    {
        println!("Kernel info:");
        //let info = &kernel.kernel_info;
        let base_info = kernel.info();
        //println!("dtb {:x} ... {}", info.dtb, some_str(&info.dtb.non_null()));
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
        /*println!(
            "kernel_guid: {:?} ... {}",
            info.kernel_guid,
            some_str(&info.kernel_guid)
        );
        println!(
            "kernel_winver: {:?} ... {}",
            info.kernel_winver.as_tuple(),
            bool_str(info.kernel_winver != (0, 0).into())
        );
        println!(
            "eprocess_base: {:x} ... {}",
            info.eprocess_base,
            some_str(&info.eprocess_base.non_null())
        );*/
        println!();
    }

    {
        if let Ok(modules) = kernel_modules(&mut kernel) {
            for k in kernel_mods.split(",") {
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
        let proc_list = kernel.process_info_list()?;
        let lsass = proc_list
            .iter()
            .find(|p| p.name.to_string().to_lowercase() == sysproc);
        println!("{} ... {}", &sysproc, some_str(&lsass));
        println!();

        if let Some(proc) = lsass {
            println!("{} info:", proc.name);
            println!("pid: {} ... {}", proc.pid, bool_str(proc.pid < 10000));
            /*let win32_proc = kernel.process_info_from_base(proc.clone())?;
            println!(
                "dtb: {} ... {}",
                win32_proc.dtb,
                some_str(&win32_proc.dtb.non_null())
            );
            println!(
                "section_base: {} ... {}",
                win32_proc.section_base,
                some_str(&win32_proc.section_base.non_null())
            );
            println!(
                "ethread: {} ... {}",
                win32_proc.ethread,
                some_str(&win32_proc.ethread.non_null())
            );
            println!(
                "teb: {:?} ... {}",
                win32_proc.teb,
                bool_str(win32_proc.teb.is_none())
            );
            println!(
                "teb_wow64: {:?} ... {}",
                win32_proc.teb_wow64,
                bool_str(win32_proc.teb_wow64.is_none())
            );
            println!(
                "peb_native: {:?} ... {}",
                win32_proc.peb_native,
                some_str(&win32_proc.peb_native)
            );
            println!(
                "peb_wow64: {:?} ... {}",
                win32_proc.teb_wow64,
                bool_str(win32_proc.peb_wow64.is_none())
            );*/
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

fn kernel_modules(kernel: &mut impl Kernel) -> Result<Vec<ModuleInfo>> {
    let modules = kernel.module_list().map_err(From::from);
    println!("kernel modules ... {}", ok_str(&modules));
    modules
}

fn build_kernel(
    mem: ConnectorInstance,
    inventory: &Inventory,
    name: &str,
    args: &Args,
) -> Result<KernelInstance> {
    let kernel = inventory.create_os(name, mem, args);
    println!("Kernel::build ... {}", ok_str(&kernel));
    println!();
    kernel
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
