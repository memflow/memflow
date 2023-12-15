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

pub mod keyboard;
pub mod module;
pub mod process;
pub mod root;
pub mod util;

pub use keyboard::{Keyboard, KeyboardState, OsKeyboard};

pub use module::{
    ExportCallback, ExportInfo, ImportCallback, ImportInfo, ModuleAddressCallback,
    ModuleAddressInfo, ModuleInfo, ModuleInfoCallback, SectionCallback, SectionInfo,
};

pub use process::{Pid, Process, ProcessInfo, ProcessInfoCallback, ProcessState};

pub use root::{Os, OsInfo};

use crate::types::Address;

use crate::cglue::*;

pub type AddressCallback<'a> = OpaqueCallback<'a, Address>;
