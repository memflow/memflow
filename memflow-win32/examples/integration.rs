use memflow::connector::*;
use memflow::mem::*;

use memflow_win32::error::{Error, Result};
use memflow_win32::win32::{Kernel, Win32ModuleInfo, Win32Process};

use clap::*;
use log::Level;

use colored::*;

static mut HAD_ERROR: bool = false;

fn main() -> Result<()> {
    let (connector, args_str) = parse_args();

    let args = ConnectorArgs::parse(&args_str)?;

    // create inventory + connector
    let inventory = unsafe { ConnectorInventory::scan() };
    let connector = unsafe { inventory.create_connector(&connector, &args)? };

    let mut kernel = build_kernel(connector)?;

    {
        println!("Kernel info:");
        let info = &kernel.kernel_info;
        let start_block = &info.start_block;
        println!(
            "{:#?} ... {}",
            start_block,
            some_str(&start_block.dtb.non_null())
        );
        println!(
            "kernel_base: {:x} ... {}",
            info.kernel_base,
            some_str(&info.kernel_base.non_null())
        );
        println!(
            "kernel_size: {:x} ... {}",
            info.kernel_size,
            bool_str(info.kernel_size != 0)
        );
        println!(
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
        );
        println!();
    }

    {
        println!("Kernel Process:");
        if let Ok(proc_info) = kernel.kernel_process_info() {
            let mut kernel_proc = Win32Process::with_kernel_ref(&mut kernel, proc_info);
            let modules = modules(&mut kernel_proc)?;
            println!("checking module list:");
            println!(
                "ntoskrnl.exe ... {}",
                some_str(
                    &modules
                        .iter()
                        .find(|e| e.name.to_lowercase() == "ntoskrnl.exe")
                )
            );
            println!(
                "hal.dll ... {}",
                some_str(&modules.iter().find(|e| e.name.to_lowercase() == "hal.dll"))
            );
        } else {
            println!("{}", bool_str(false));
        }
        println!();
    }

    {
        println!("Process List:");
        let proc_list = kernel.process_info_list()?;
        let lsass = proc_list
            .iter()
            .find(|p| p.name.to_lowercase() == "lsass.exe");
        println!("lsass.exe ... {}", some_str(&lsass));
        println!();

        if let Some(proc) = lsass {
            println!("{} info:", proc.name);
            println!("pid: {} ... {}", proc.pid, bool_str(proc.pid < 10000));
            println!("dtb: {} ... {}", proc.dtb, some_str(&proc.dtb.non_null()));
            println!(
                "section_base: {} ... {}",
                proc.section_base,
                some_str(&proc.section_base.non_null())
            );
            println!(
                "ethread: {} ... {}",
                proc.ethread,
                some_str(&proc.ethread.non_null())
            );
            println!("teb: {:?} ... {}", proc.teb, bool_str(proc.teb.is_none()));
            println!(
                "teb_wow64: {:?} ... {}",
                proc.teb_wow64,
                bool_str(proc.teb_wow64.is_none())
            );
            println!(
                "peb_native: {} ... {}",
                proc.peb_native,
                some_str(&proc.peb_native.non_null())
            );
            println!(
                "peb_wow64: {:?} ... {}",
                proc.teb_wow64,
                bool_str(proc.peb_wow64.is_none())
            );
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

fn modules<V: VirtualMemory>(process: &mut Win32Process<V>) -> Result<Vec<Win32ModuleInfo>> {
    let modules = process.module_list();
    println!("modules ... {}", ok_str(&modules));
    modules
}

fn build_kernel<T: PhysicalMemory>(
    mem: T,
) -> Result<Kernel<impl PhysicalMemory, impl VirtualTranslate>> {
    let kernel = Kernel::builder(mem).build_default_caches().build();
    println!("Kernel::build ... {}", ok_str(&kernel));
    println!();
    kernel
}

fn parse_args() -> (String, String) {
    let matches = App::new("read_keys example")
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
        matches.value_of("args").unwrap().into(),
    )
}
