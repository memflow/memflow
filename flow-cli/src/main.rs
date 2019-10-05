use clap::{App, Arg};

use flow_qemu::BridgeConnector;
use flow_win32;

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

    // os functionality should be located in core!
    flow_win32::init(&mut bridge).unwrap();
}
