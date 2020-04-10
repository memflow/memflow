mod init;
use init::*;

#[macro_use]
extern crate clap;
use clap::{App, ArgMatches};

use log::Level;

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

    match argv.value_of("connector").unwrap_or_else(|| "bridge") {
        "bridge" => {
            let mut conn = init_bridge::init_bridge(&argv).unwrap();
            run(&argv, &mut conn)
        }
        "qemu_procfs" => {
            let mut conn = init_qemu_procfs::init_qemu_procfs().unwrap();
            run(&argv, &mut conn)
        }
        _ => Err(Error::new("the connector requested does not exist")),
    }
}

fn run<T>(argv: &ArgMatches, conn: &mut T) -> Result<()>
where
    T: PhysicalMemoryTrait + VirtualMemoryTrait,
{
    // TODO: osname from config/params?
    //let connrc = Rc::new(RefCell::new(conn));
    let os = match argv.value_of("os").unwrap_or_else(|| "win32") {
        "win32" => Win32::try_with(conn),
        //"linux" => {},
        _ => Err(flow_win32::error::Error::new("invalid os")),
    }
    .unwrap();

    let offsets = Win32Offsets::try_with_guid(&os.kernel_guid()).unwrap();
    let eprocs = os.eprocess_list(conn, &offsets).unwrap();
    eprocs
        .iter()
        .map(|eproc| Win32UserProcess::try_with_eprocess(conn, &os, &offsets, *eproc))
        .filter_map(std::result::Result::ok)
        .for_each(|p| println!("{:?} {:?}", p.pid(), p.name()));

    let csgo = Win32UserProcess::try_with_name(conn, &os, &offsets, "csgo.exe").unwrap();
    println!("csgo found: {:?}", csgo);

    let pebs = csgo.peb_list(conn).unwrap();
    pebs.iter()
        .map(|peb| Win32Module::try_with_peb(conn, &csgo, &offsets, *peb))
        .filter_map(std::result::Result::ok)
        .for_each(|module| println!("{:?} {:?}", module.base(), module.name()));

    /*
    let kernel = Win32KernelProcess::try_with(conn, &os).unwrap();
    println!("kernel found: {:?}", kernel);
    kernel
        .peb_list(conn, &offsets)
        .unwrap()
        .iter()
        .map(|peb| Win32Module::try_with_peb(conn, &calc, &offsets, *peb))
        .filter_map(std::result::Result::ok)
        .for_each(|module| println!("{:?} {:?}", module.base(), module.name()));
        */

    Ok(())
}
