use clap::{App, Arg};
use pretty_env_logger;
use std::io::{Error, ErrorKind, Result};
use std::path::PathBuf;
use std::rc::Rc;
use std::cell::RefCell;

use flow_core::address::{Address, Length};
use flow_core::mem::VirtualRead;
use flow_qemu::BridgeConnector;
use flow_win32;
use flow_win32::cache;
use flow_win32::win::{Windows, process::Process};
use goblin::pe::{options::ParseOptions, PE};
use pdb::{FallibleIterator, PdbInternalSectionOffset};

fn main() {
    pretty_env_logger::init();

    let argv = App::new("flow-core")
        .version("0.1")
        .arg(
            Arg::with_name("socket")
                .short("s")
                .long("socket")
                .value_name("FILE")
                .help("bridge unix socket file")
                .takes_value(true),
        )
        .get_matches();

    // this is just some test code
    let socket = argv
        .value_of("socket")
        .unwrap_or("/tmp/qemu-connector-bridge");
    let bridge = match BridgeConnector::connect(socket) {
        Ok(s) => s,
        Err(e) => {
            println!("couldn't connect to bridge: {:?}", e);
            return;
        }
    };

    // os functionality should be located in core!
    let bridgerc = Rc::new(RefCell::new(bridge));
    let mut win = flow_win32::init(bridgerc).unwrap();

    let mut proc = win
        .process_iter()
        .filter_map(|mut p| {
            if p.get_name().unwrap_or_default() == "svchost.exe" {
                Some(p)
            } else {
                None
            }
        })
        .nth(0)
        .ok_or_else(|| "unable to find svchost.exe")
        .unwrap();

    println!("found svchost.exe: {:?} {:?}", proc.get_pid(), proc.get_name());
}
