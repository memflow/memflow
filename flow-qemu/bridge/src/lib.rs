// Prevent a spurious 'unused_imports' warning
#[allow(unused_imports)]
#[macro_use]
extern crate ctor;

#[macro_use]
extern crate lazy_static;

use libc_print::*;

use rand::{self, distributions::Alphanumeric, Rng};
use std::{env, thread};

#[macro_use]
mod native;

mod cpu;
mod mem;
mod rpc;

#[allow(dead_code)]
mod bridge_capnp {
    include!(concat!(env!("OUT_DIR"), "/bridge_capnp.rs"));
}

lazy_static! {
    static ref BR_SOCKET: String = {
        env::var("BR_SOCKET").unwrap_or_else(|_e| {
            "/tmp/qemu-connector-bridge-".to_string()
                + &rand::thread_rng()
                    .sample_iter(&Alphanumeric)
                    .take(8)
                    .collect::<String>()
                    .to_lowercase()
        })
    };
}

#[ctor]
fn construct() {
    let socket = &*BR_SOCKET;
    libc_eprintln!("starting qemu-connector-bridge at {}", socket);
    thread::spawn(move || {
        rpc::listen(socket).unwrap();
    });
}

#[dtor]
fn destruct() {
    // TODO: verify if the socket was created properly!
    std::fs::remove_file(&*BR_SOCKET).unwrap();
}
