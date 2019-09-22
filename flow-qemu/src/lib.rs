pub mod bridge;
pub mod monitor;
pub mod qmp;

#[allow(dead_code)]
mod bridge_capnp {
    include!(concat!(env!("OUT_DIR"), "/bridge_capnp.rs"));
}

pub use self::bridge::BridgeConnector;
pub use self::monitor::QemuMonitor;
pub use self::qmp::Qmp;
