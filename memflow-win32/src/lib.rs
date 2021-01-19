/*!
This crate contains memflow's win32 implementation.
It is used to interface with windows targets.
*/

#![cfg_attr(not(feature = "std"), no_std)]
extern crate no_std_compat as std;

pub mod error;

pub mod kernel;

pub mod offsets;

pub mod win32;

pub mod prelude {
    pub mod v1 {
        pub use crate::error::*;
        pub use crate::kernel::*;
        pub use crate::offsets::*;
        pub use crate::win32::*;
    }
    pub use v1::*;
}

#[cfg(feature = "plugins")]
mod plugin {
    use memflow::derive::os_layer;
    use memflow::os::Kernel;
    use memflow::plugins::{Args, ConnectorInstance};

    #[os_layer(name = "win32")]
    pub fn build_kernel<'a>(
        _args: &Args,
        mem: ConnectorInstance,
        log_level: log::Level,
    ) -> memflow::error::Result<impl Kernel<'a> + Clone> {
        simple_logger::SimpleLogger::new()
            .with_level(log_level.to_level_filter())
            .init()
            .ok();
        crate::win32::Win32Kernel::builder(mem)
            .build_default_caches()
            .build()
            .map_err(From::from)
    }
}
