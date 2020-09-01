/*!
This crate contains memflow's win32 implementation.
It is used to interface with windows targets.
*/

#![cfg_attr(not(feature = "std"), no_std)]
extern crate no_std_compat as std;

pub mod error;
#[doc(hidden)]
pub use error::*;

pub mod kernel;
#[doc(hidden)]
pub use kernel::*;

pub mod offsets;
#[doc(hidden)]
pub use offsets::*;

pub mod win32;
#[doc(hidden)]
pub use win32::*;
