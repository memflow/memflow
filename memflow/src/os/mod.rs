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
