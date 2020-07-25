/*!
This crate contains memflow's win32 implementation.
It is used to interface with windows targets.
*/

#![cfg_attr(not(feature = "std"), no_std)]
extern crate no_std_compat as std;

pub mod error;
pub use error::*;

// TODO: private these
pub mod kernel;
pub use kernel::*;

pub mod pe;
pub use pe::*; // TODO: restrict forwarding

// TODO: enable again
//pub mod keyboard;
//pub use keyboard::*;

pub mod offsets;
pub use offsets::*;

pub mod win32;
pub use win32::*;
