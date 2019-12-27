mod config;
mod init;

#[macro_use]
extern crate clap;
use clap::App;

use simple_logger;
use log::Level;
use std::cell::RefCell;
use std::rc::Rc;

use flow_core::*;
use flow_core::connector::qemu_procfs;

use flow_win32;
use flow_win32::win::process::ProcessModuleTrait; // TODO: import in flow_win32

// TODO: this is os agnostic and just temp?
use flow_win32::keyboard::Keyboard;

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

    //let conn = init::init_connector(&argv).unwrap();
    let conn = qemu_procfs::Memory::new().unwrap();

    // TODO: osname from config/params?
    let connrc = Rc::new(RefCell::new(conn));
    let os = match argv.value_of("os").unwrap_or_else(|| "win32") {
        "win32" => flow_win32::init(connrc),
        //"linux" => {},
        _ => Err(flow_win32::error::Error::new("invalid os")),
    }
    .unwrap();

    match argv.subcommand() {
        ("kernel", Some(kernel_matches)) => match kernel_matches.subcommand() {
            ("module", Some(module_matches)) => match module_matches.subcommand() {
                ("ls", Some(_)) => {
                    os.kernel_process()
                        .unwrap()
                        .module_iter()
                        .unwrap()
                        .for_each(|mut m| {
                            if let Ok(name) = m.name() {
                                println!("{}", name);
                            }
                        });
                }
                _ => println!("invalid command {:?}", module_matches),
            },
            _ => println!("invalid command {:?}", kernel_matches),
        },
        ("process", Some(kernel_matches)) => {
            match kernel_matches.subcommand() {
                ("ls", Some(_)) => {
                    os.process_iter().for_each(|mut m| {
                        if let Ok(pid) = m.pid() {
                            if let Ok(name) = m.name() {
                                println!("{} {}", pid, name);
                            }
                        }
                    });
                }
                ("module", Some(module_matches)) => {
                    match module_matches.subcommand() {
                        ("ls", Some(_process)) => {
                            println!("test1") // TODO: specify process name/pid
                        }
                        _ => println!("invalid command {:?}", module_matches),
                    }
                }
                _ => println!("invalid command {:?}", kernel_matches),
            }
        }
        ("keylog", Some(_)) => {
            println!("keylogging");
            let mut kbd = Keyboard::with(&os).unwrap();
            loop {
                let kbs = kbd.state().unwrap();
                if kbs.down(win_key_codes::VK_LBUTTON).unwrap() {
                    println!("VK_LBUTTON down");
                } else {
                    println!("VK_LBUTTON up");
                }
            }
        }
        ("", None) => println!("no command specified"),
        _ => println!("invalid command {:?}", argv),
    }

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
