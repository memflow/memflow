#[macro_use]
pub mod address;

pub mod arch;
pub mod mem;
pub mod vat;

pub mod machine;
pub mod os;

pub mod cpu;
pub mod net;

// bridge
// TODO: can we put this in bridge somehow?
#[allow(dead_code)]
mod bridge_capnp {
    include!(concat!(env!("OUT_DIR"), "/bridge_capnp.rs"));
}

pub mod bridge;
