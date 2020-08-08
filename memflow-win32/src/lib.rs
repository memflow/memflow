/*!
This crate contains memflow's win32 implementation.
It is used to interface with windows targets.
*/

#![cfg_attr(not(feature = "std"), no_std)]
extern crate no_std_compat as std;

pub mod error;
#[doc(hidden)]
pub use error::*;

// TODO: private these
pub mod kernel;
#[doc(hidden)]
pub use kernel::*;

// TODO: feature gate pelite + maybe add goblin
pub mod pe;
#[doc(hidden)]
pub use pe::*; // TODO: restrict forwarding

// TODO: enable again
//pub mod keyboard;
//pub use keyboard::*;

pub mod offsets;
#[doc(hidden)]
pub use offsets::*;

pub mod win32;
#[doc(hidden)]
pub use win32::*;
