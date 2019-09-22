use clap::{App, Arg};
use std::time::Instant;

use flow_core::mem::PhysicalMemory;
use flow_qemu::BridgeConnector;
use flow_win32::dtb;

fn main() {
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

    dtb::find(&mut bridge).unwrap();
}
