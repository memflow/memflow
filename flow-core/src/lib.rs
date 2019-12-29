pub mod error;
pub use error::{Error, Result};

#[macro_use]
pub mod address;
pub use address::{Address, Length};

pub mod arch;
pub use arch::{Architecture, ArchitectureTrait, ByteOrder, InstructionSet};

pub mod mem;
pub use mem::*;

pub mod vat;
pub use vat::{VatImpl, VirtualAddressTranslation};

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
