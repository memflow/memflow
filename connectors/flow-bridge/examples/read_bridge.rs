use clap::{App, Arg};
use std::time::Instant;

use log::info;

use flow_bridge::BridgeClient;
use flow_core::*;

fn main() {
    let argv = App::new("examples/bridge_read")
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

    let url = argv
        .value_of("bridge-url")
        .unwrap_or("unix:/tmp/qemu-connector-bridge");
    let mut bridge = match BridgeClient::connect(url) {
        Ok(s) => s,
        Err(e) => {
            info!("couldn't connect to bridge: {:?}", e);
            return;
        }
    };

    let mut mem = vec![0; 8];
    bridge
        .phys_read_raw_into(Address::from(0x1000).into(), &mut mem)
        .unwrap();
    info!("Received memory: {:?}", mem);

    let start = Instant::now();
    let mut counter = 0;
    loop {
        let mut buf = vec![0; 0x1000];
        bridge
            .phys_read_raw_into(Address::from(0x1000).into(), &mut buf)
            .unwrap();

        counter += 1;
        if (counter % 10000) == 0 {
            let elapsed = start.elapsed().as_millis() as f64;
            if elapsed > 0.0 {
                info!("{} reads/sec", (f64::from(counter)) / elapsed * 1000.0);
                info!(
                    "{} reads/frame",
                    (f64::from(counter)) / elapsed * 1000.0 / 60.0
                );
                info!("{} ms/read", elapsed / (f64::from(counter)));
            }
        }
    }
}
