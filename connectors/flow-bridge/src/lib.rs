#[allow(dead_code)]
mod bridge_capnp {
    include!(concat!(env!("OUT_DIR"), "/bridge_capnp.rs"));
}

pub mod server;
pub use server::BridgeServer;

pub mod client;
pub use client::BridgeClient;
