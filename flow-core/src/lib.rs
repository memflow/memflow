pub mod error;
pub use error::*;

#[macro_use]
pub mod address;
pub use address::*;

pub mod arch;
pub use arch::*;

pub mod mem;
pub use mem::*;

pub mod vat;
pub use vat::*;

pub mod process;
pub use process::*;

//pub mod cpu;
//pub mod net;

// bridge
// TODO: can we put this in bridge somehow?
#[allow(dead_code)]
mod bridge_capnp {
    include!(concat!(env!("OUT_DIR"), "/bridge_capnp.rs"));
}

pub mod connector;

pub mod ida_pattern;
pub use ida_pattern::*;
