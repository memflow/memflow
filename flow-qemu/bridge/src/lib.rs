// Prevent a spurious 'unused_imports' warning
#[allow(unused_imports)]
#[macro_use]
extern crate ctor;

#[macro_use]
extern crate lazy_static;

use libc_print::*;

use rand::{self, distributions::Alphanumeric, Rng};
use std::{env, thread};
use url::Url;

#[macro_use]
mod native;

mod cpu;
mod mem;
mod rpc;

#[allow(dead_code)]
mod bridge_capnp {
    include!(concat!(env!("OUT_DIR"), "/bridge_capnp.rs"));
}

// <qemu:env name="BRIDGE_ADDRESS" value="unix:/tmp/qemu-connector-bridge-win10"/>
// <qemu:env name="BRIDGE_ADDRESS" value="tcp:127.0.0.1:8181"/>
lazy_static! {
    static ref BRIDGE_ADDRESS: String = {
        env::var("BRIDGE_ADDRESS").unwrap_or_else(|_e| {
            "/tmp/qemu-connector-bridge-".to_string()
                + &rand::thread_rng()
                    .sample_iter(&Alphanumeric)
                    .take(8)
                    .collect::<String>()
                    .to_lowercase()
        })
    };
}

// we dont wanna execute construct/destruct in tests
#[cfg(not(test))]
#[ctor]
fn construct() {
    let addr = &*BRIDGE_ADDRESS;
    libc_eprintln!("starting qemu-connector-bridge at {}", addr);
    thread::spawn(move || {
        // TODO: retry + timeout?
        match rpc::listen(addr) {
            Ok(_) => (),
            Err(e) => {
                libc_eprintln!("unable to start qemu-connector-bridge: {:?}", e);
            }
        }
    });
}

#[cfg(not(test))]
#[dtor]
fn destruct() {
    // TODO: verify if the socket was created properly!
    let addr = &*BRIDGE_ADDRESS;
    if let Ok(u) = Url::parse(addr) {
        if u.scheme() == "unix" {
            std::fs::remove_file(u.path()).unwrap();
        }
    }
}
