use clap::{App, AppSettings, Arg};
use pretty_env_logger;
use std::cell::RefCell;
use std::rc::Rc;

use flow_core::bridge::client::BridgeClient;
use flow_win32;
use flow_win32::win::process::ProcessTrait; // TODO: import in flow_win32

fn main() {
    pretty_env_logger::init();

    let argv = App::new("flow-core")
        .version("0.1")
        .arg(
            Arg::with_name("url")
                .short("url")
                .long("url")
                .value_name("URL")
                .help("socket url")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("os")
                .short("os")
                .long("os")
                .value_name("OS")
                .help("target operating system")
                .takes_value(true),
        )
        .subcommand(
            App::new("kernel")
                .about("os kernel specific options")
                .setting(AppSettings::SubcommandRequiredElseHelp)
                .subcommand(
                    App::new("module")
                        .about("kernel module options")
                        .subcommand(App::new("ls").about("lists loaded kernel modules")),
                ),
        )
        .subcommand(
            App::new("process")
                .about("os process specific options")
                .setting(AppSettings::SubcommandRequiredElseHelp)
                .subcommand(App::new("ls").about("lists active processes")),
        )
        .get_matches();

    // this is just some test code
    let url = argv.value_of("url").unwrap_or("unix:/tmp/memflow-bridge");
    let bridge = match BridgeClient::connect(url) {
        Ok(br) => br,
        Err(e) => {
            println!("couldn't connect to bridge: {:?}", e);
            return;
        }
    };

    // os functionality should be located in core!
    let bridgerc = Rc::new(RefCell::new(bridge));

    let os = match argv.value_of("os").unwrap_or("win32") {
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
                        ("ls", Some(_)) => {
                            println!("test1") // TODO: specify process name/pid
                        }
                        _ => println!("invalid command {:?}", module_matches),
                    }
                }
                _ => println!("invalid command {:?}", kernel_matches),
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
