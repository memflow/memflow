use clap::{App, Arg};
use pretty_env_logger;
use std::cell::RefCell;
use std::rc::Rc;

use flow_core::bridge::client::BridgeClient;
use flow_win32;

fn main() {
    pretty_env_logger::init();

    let argv = App::new("flow-core")
        .version("0.1")
        .arg(
            Arg::with_name("bridge-url")
                .short("url")
                .long("bridge-url")
                .value_name("URL")
                .help("bridge socket url")
                .takes_value(true),
        )
        .get_matches();

    // this is just some test code
    let url = argv
        .value_of("bridge-url")
        .unwrap_or("unix:/tmp/qemu-connector-bridge");
    let bridge = match BridgeClient::connect(url) {
        Ok(br) => br,
        Err(e) => {
            println!("couldn't connect to bridge: {:?}", e);
            return;
        }
    };

    // os functionality should be located in core!
    let bridgerc = Rc::new(RefCell::new(bridge));
    let win = flow_win32::init(bridgerc).unwrap();

    win.process_iter()
        .for_each(|mut p| println!("{:?} {:?}", p.get_pid(), p.get_name()));
    win.process_iter()
        .for_each(|mut p| println!("{:?} {:?}", p.get_pid(), p.get_name()));

    let mut process = win
        .process_iter()
        .filter_map(|mut p| {
            if p.get_name().unwrap_or_default() == "Calculator.exe" {
                Some(p)
            } else {
                None
            }
        })
        .nth(0)
        .ok_or_else(|| "unable to find Calculator.exe")
        .unwrap();

    println!(
        "found Calculator.exe: {:?} {:?} {:?}",
        process.get_pid(),
        process.get_name(),
        process.is_wow64()
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
            if m.get_name().unwrap_or_default() == "Calculator.exe" {
                Some(m)
            } else {
                None
            }
        })
        .nth(0)
        .ok_or_else(|| "unable to find module in Calculator.exe")
        .unwrap();

    println!("mod: {:?}", module.clone().get_name());
}
