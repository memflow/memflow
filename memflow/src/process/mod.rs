/*!
Traits for OS independent process abstractions.
*/

use std::prelude::v1::*;

use crate::architecture::ArchitectureObj;
use crate::types::Address;

/// Trait describing a operating system
pub trait OperatingSystem {}

/// Type alias for a PID.
pub type PID = u32;

/// Trait describing OS independent process information.
pub trait OsProcessInfo {
    /// Returns the base address of this process.
    ///
    /// # Remarks
    ///
    /// On Windows this will return the address of the [`_EPROCESS`](https://www.nirsoft.net/kernel_struct/vista/EPROCESS.html) structure.
    fn address(&self) -> Address;

    /// Returns the pid of this process.
    fn pid(&self) -> PID;

    /// Returns the name of the process.
    ///
    /// # Remarks
    ///
    /// On Windows this will be clamped to 16 characters.
    fn name(&self) -> String;

    /// Returns the architecture of the target system.
    fn sys_arch(&self) -> ArchitectureObj;

    /// Returns the architecture of the process.
    ///
    /// # Remarks
    ///
    /// Specifically on 64-bit systems this could be different
    /// to the `sys_arch` in case the process is an emulated 32-bit process.
    ///
    /// On windows this technique is called [`WOW64`](https://docs.microsoft.com/en-us/windows/win32/winprog64/wow64-implementation-details).
    fn proc_arch(&self) -> ArchitectureObj;
}

// TODO: Exports / Sections / etc
