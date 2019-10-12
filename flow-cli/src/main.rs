use clap::{App, Arg};
use pretty_env_logger;
use std::io::{Error, ErrorKind, Result};

use address::{Address, Length};
use flow_qemu::BridgeConnector;
use flow_win32;
use flow_win32::cache;
use flow_win32::win::Windows;
use goblin::pe::{options::ParseOptions, PE};
use mem::VirtualRead;

fn microsoft_download_ntos<T: VirtualRead>(mem: &mut T, win: &Windows) -> Result<()> {
    let ntos_buf = mem
        .virt_read(
            win.dtb.arch,
            win.dtb.dtb,
            win.kernel_base,
            Length::from_mb(32),
        )
        .unwrap();

    let mut pe_opts = ParseOptions::default();
    pe_opts.resolve_rva = false;

    let pe = match PE::parse_with_opts(&ntos_buf, &pe_opts) {
        Ok(pe) => {
            //println!("find_x64_with_va: found pe header:\n{:?}", pe);
            pe
        }
        Err(e) => {
            return Err(Error::new(ErrorKind::Other, "unable to parse pe header"));
        }
    };

    cache::fetch_pdb(&pe).unwrap();

    Ok(())
}

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
    let mut bridge = match BridgeConnector::connect(socket) {
        Ok(s) => s,
        Err(e) => {
            println!("couldn't connect to bridge: {:?}", e);
            return;
        }
    };

    // os functionality should be located in core!
    let win = flow_win32::init(&mut bridge).unwrap();
    microsoft_download_ntos(&mut bridge, &win).unwrap();
}
