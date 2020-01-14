mod derive;
mod init;

#[macro_use]
extern crate clap;
use clap::App;

use log::Level;
//use simple_logger;
use std::cell::RefCell;
use std::rc::Rc;

use flow_core::*;

use flow_win32;
use flow_win32::*;
//use flow_win32::win::process::*;

/*
fn handle_argv<T: VirtualRead>(argv: &clap::ArgMatches, os: &flow_win32::Windows<T>) -> () {
    match argv.subcommand() {
        ("kernel", Some(kernel_matches)) => match kernel_matches.subcommand() {
            ("module", Some(module_matches)) => match module_matches.subcommand() {
                ("ls", Some(_)) => {
                    println!("base size name");
                    os.kernel_process()
                        .unwrap()
                        .module_iter()
                        .unwrap()
                        .for_each(|mut m| {
                            println!(
                                "0x{:x} 0x{:x} {}",
                                m.base().unwrap_or_default(),
                                m.size().unwrap_or_default(),
                                m.name().unwrap_or_else(|_| "{error}".to_owned())
                            )
                        });
                }
                ("export", Some(export_matches)) => match export_matches.subcommand() {
                    ("ls", Some(ls_matches)) => {
                        let prc = os.kernel_process().unwrap();
                        let mut md = prc
                            .module(ls_matches.value_of("module_name").unwrap())
                            .unwrap();
                        println!("offset rva size name");
                        md.exports().unwrap().iter().for_each(|e| {
                            println!("0x{:x} {}", e.offset, e.name);
                        });
                    }
                    _ => println!("invalid command {:?}", export_matches),
                },
                ("section", Some(section_matches)) => match section_matches.subcommand() {
                    ("ls", Some(ls_matches)) => {
                        let prc = os.kernel_process().unwrap();
                        let mut md = prc
                            .module(ls_matches.value_of("module_name").unwrap())
                            .unwrap();
                        println!("virt_addr virt_size size_of_raw_data characteristics name");
                        /*
                        md.sections().unwrap().iter().for_each(|s| {
                            println!(
                                "0x{:x} 0x{:x} 0x{:x} 0x{:x} {}",
                                s.virt_addr,
                                s.virt_size,
                                s.size_of_raw_data,
                                s.characteristics,
                                s.name
                            );
                        });*/
                    }
                    _ => println!("invalid command {:?}", section_matches),
                },
                _ => println!("invalid command {:?}", module_matches),
            },
            _ => println!("invalid command {:?}", kernel_matches),
        },
        ("process", Some(kernel_matches)) => match kernel_matches.subcommand() {
            ("ls", Some(_)) => {
                println!("pid name");
                os.process_iter().for_each(|mut p| {
                    println!(
                        "{} {}",
                        p.pid().unwrap_or_default(),
                        p.name().unwrap_or_else(|_| "{error}".to_owned())
                    );
                });
            }
            ("module", Some(module_matches)) => match module_matches.subcommand() {
                ("ls", Some(ls_matches)) => {
                    let prc = os
                        .process(ls_matches.value_of("process_name").unwrap())
                        .unwrap();
                    println!("base size name");
                    prc.module_iter().unwrap().for_each(|mut m| {
                        println!(
                            "0x{:x} 0x{:x} {}",
                            m.base().unwrap_or_default(),
                            m.size().unwrap_or_default(),
                            m.name().unwrap_or_else(|_| "{error}".to_owned())
                        )
                    });
                }
                ("export", Some(export_matches)) => match export_matches.subcommand() {
                    ("ls", Some(ls_matches)) => {
                        let prc = os
                            .process(ls_matches.value_of("process_name").unwrap())
                            .unwrap();
                        let mut md = prc
                            .module(ls_matches.value_of("module_name").unwrap())
                            .unwrap();
                        println!("offset rva size name");
                        md.exports().unwrap().iter().for_each(|e| {
                            println!("0x{:x} {}", e.offset, e.name);
                        });
                    }
                    _ => println!("invalid command {:?}", export_matches),
                },
                _ => println!("invalid command {:?}", module_matches),
            },
            _ => println!("invalid command {:?}", kernel_matches),
        },
        ("", None) => println!("no command specified"),
        _ => println!("invalid command {:?}", argv),
    }
}*/

fn main() {
    let yaml = load_yaml!("cli.yml");
    let argv = App::from(yaml).get_matches();

    match argv.occurrences_of("verbose") {
        1 => simple_logger::init_with_level(Level::Warn).unwrap(),
        2 => simple_logger::init_with_level(Level::Info).unwrap(),
        3 => simple_logger::init_with_level(Level::Debug).unwrap(),
        4 => simple_logger::init_with_level(Level::Trace).unwrap(),
        _ => simple_logger::init_with_level(Level::Error).unwrap(),
    }

    // TODO: command for connector?

    // TODO: feature
    let mut conn = init::init_connector(&argv).unwrap();

    // TODO: osname from config/params?
    //let connrc = Rc::new(RefCell::new(conn));
    let os = match argv.value_of("os").unwrap_or_else(|| "win32") {
        "win32" => flow_win32::init(&mut conn),
        //"linux" => {},
        _ => Err(flow_win32::error::Error::new("invalid os")),
    }
    .unwrap();

    let offsets = Win32Offsets::try_with_guid(&os.kernel_guid()).unwrap();
    let eprocs = os.eprocess_list(&mut conn, &offsets).unwrap();
    eprocs
        .iter()
        .map(|eproc| Win32UserProcess::try_with_eprocess(&mut conn, &os, &offsets, *eproc))
        .filter_map(std::result::Result::ok)
        .for_each(|p| println!("{:?} {:?}", p.pid(), p.name()));

    let calc = Win32UserProcess::try_with_name(&mut conn, &os, &offsets, "Calculator.exe").unwrap();
    println!("calc found: {}", calc.pid());

    let pebs = calc.peb_list(&mut conn, &offsets).unwrap();
    pebs
    .iter()
    .map(|peb| Win32Module::try_with_peb(&mut conn, &calc, &offsets, *peb))
    .filter_map(std::result::Result::ok)
    .for_each(|module| println!("{:?} {:?}", module.base(), module.name()));

    // init offsets with guid (+autodownload)

    // TODO: interactive/non-interactive mode switch
    //handle_argv(&argv, &os);

    // derive test
    /*
    let mut vrw = derive::VirtualReadWriteDerive {};
    vrw.virt_read(
        Architecture::from(InstructionSet::X64),
        addr!(0),
        addr!(0),
        len!(0),
    )
    .unwrap();
    vrw.virt_write(
        Architecture::from(InstructionSet::X64),
        addr!(0),
        addr!(0),
        &Vec::new(),
    )
    .unwrap();
    */

    // parse kernel pe header -- start
    /*use flow_core::error::*;

    */
    //println!("module_list: {:x}", module_list);
    // parse kernel pe header -- end

    /*
        win.process_iter()
            .for_each(|mut p| println!("{:?} {:?}", p.get_pid(), p.get_name()));
        win.process_iter()
            .for_each(|mut p| println!("{:?} {:?}", p.get_pid(), p.get_name()));
    */
    /*
    let mut process = win
        .process_iter()
        .filter_map(|mut p| {
            if p.name().unwrap_or_default() == "Steam.exe" {
                Some(p)
            } else {
                None
            }
        })
        .nth(0)
        .ok_or_else(|| "unable to find Steam.exe")
        .unwrap();

    println!(
        "found Steam.exe: {:?} {:?} {:?}",
        process.pid(),
        process.name(),
        process.has_wow64()
    );
        process
            .module_iter()
            .unwrap()
            .for_each(|mut m| println!("{:?}", m.get_name()));
        process
            .module_iter()
            .unwrap()
            .for_each(|mut m| println!("{:?}", m.get_name()));
    let module = process
        .module_iter()
        .unwrap()
        .filter_map(|mut m| {
            if m.name().unwrap_or_default() == "Steam.exe" {
                Some(m)
            } else {
                None
            }
        })
        .nth(0)
        .ok_or_else(|| "unable to find module in Calculator.exe")
        .unwrap();

    println!("mod: {:?}", module.clone().name());
    */
}
