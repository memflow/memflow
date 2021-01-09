//pub mod inventory;
pub mod kernel;
pub mod module;
pub mod process;

pub use kernel::Kernel;
pub use module::{ModuleAddressCallback, ModuleAddressInfo, ModuleInfo, ModuleInfoCallback};
pub use process::{Process, ProcessInfo, ProcessInfoCallback, PID};

use crate::types::{Address, OpaqueCallback};
pub type AddressCallback<'a, T> = OpaqueCallback<'a, T, Address>;
