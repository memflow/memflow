//! Describes an operating system in high level.
//!
//! Currently there are 3 key parts describing an OS, each subsetting the previous level:
//! * `OS`
//! * `Process`
//! * `ModuleInfo`
//!
//! `OS` abstracts away the very root of the system. Often times, the underlying object is a OS
//! kernel, but it should not be a concern, because it is designed to also work with various non-OS
//! systems like UEFI firmware, as well as pseudo implementations that use native system calls.
//!
//! `Process` abstracts away a single process. It provides memory access, module lists, and more.
//!
//! `ModuleInfo` currently is just an information block, without any memory access, or special
//! functions. It might be wise to implement helpers for exported functions, memory protection
//! flags, and other things concerned with individual modules.

pub mod module;
pub mod process;
pub mod system;

pub use module::{ModuleAddressCallback, ModuleAddressInfo, ModuleInfo, ModuleInfoCallback};
pub use process::{Process, ProcessInfo, ProcessInfoCallback, PID};
pub use system::{OSInfo, OSInner, OS};

use crate::types::{Address, OpaqueCallback};
pub type AddressCallback<'a> = OpaqueCallback<'a, Address>;

/// # Safety
///
/// No safety to be found. MAKE SURE YOU DON'T ALIAS THE POINTERS!!!!
pub(crate) unsafe fn clone_self<T: ?Sized>(m: &mut T) -> (&mut T, &mut T) {
    let p = m as *mut T;
    (p.as_mut().unwrap(), p.as_mut().unwrap())
}
