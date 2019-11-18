#[allow(dead_code)]
mod bridge_capnp {
    include!(concat!(env!("OUT_DIR"), "/bridge_capnp.rs"));
}

pub mod server;
pub mod client;
