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

#[deprecated]
pub use prelude::v1::*;
