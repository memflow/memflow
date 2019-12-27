mod config;

#[macro_use]
extern crate clap;
use clap::App;

use pretty_env_logger;
use std::cell::RefCell;
use std::rc::Rc;

use flow_core::connector::bridge::client::BridgeClient;
use flow_core::connector::qemu_procfs;
use flow_core::*;
use flow_win32;
use flow_win32::win::process::ProcessModuleTrait; // TODO: import in flow_win32

// TODO: this is os agnostic and just temp?
use flow_win32::keyboard::Keyboard;

fn main() {
    pretty_env_logger::init();

    let yaml = load_yaml!("cli.yml");
    let argv = App::from(yaml).get_matches();

    // if url && os {} else { config set? else auto conf }
    let (url, osname) = {
        if argv.is_present("url") {
            (
                argv.value_of("url").unwrap().to_owned(),
                argv.value_of("os").unwrap_or_else(|| "win32").to_owned(),
            )
        } else {
            let machines =
                config::try_parse(argv.value_of("config").unwrap_or_else(|| "memflow.toml"))
                    .unwrap()
                    .machine
                    .unwrap(); // TODO: proper error handling / feedback

            let machine = {
                if argv.is_present("machine") {
                    machines
                        .iter()
                        .filter(|m| m.name.as_ref().unwrap() == argv.value_of("machine").unwrap())
                        .nth(0)
                        .ok_or_else(|| {
                            std::io::Error::new(std::io::ErrorKind::Other, "machine not found")
                        })
                } else if machines.len() == 1 {
                    Ok(&machines[0])
                } else {
                    Err(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        "no machine specified",
                    ))
                }
            }
            .unwrap();

            (
                machine.url.to_owned().unwrap(),
                machine
                    .os
                    .to_owned()
                    .unwrap_or_else(|| String::from("win32")),
            )
        }
    };

    // TODO: make this configurable via cli
    let bridge = match qemu_procfs::Memory::new() {
        Ok(br) => br,
        Err(e) => {
            println!("couldn't open memory read context: {:?}", e);
            return;
        }
    };

    /*
    let bridge = match BridgeClient::connect(url.as_str()) {
        Ok(br) => br,
        Err(e) => {
            println!("couldn't connect to bridge: {:?}", e);
            return;
        }
    };
    */

    let bridgerc = Rc::new(RefCell::new(bridge));
    let os = match osname.as_str() {
        "win32" => flow_win32::init(bridgerc),
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
