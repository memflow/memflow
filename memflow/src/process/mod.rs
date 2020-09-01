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

// TODO: Range impl for base to size?
/// Trait describing OS independent module information.
pub trait OsProcessModuleInfo {
    /// Returns the address of the module header.
    ///
    /// # Remarks
    ///
    /// On Windows this will return the address where the [`PEB`](https://docs.microsoft.com/en-us/windows/win32/api/winternl/ns-winternl-peb) entry is stored.
    fn address(&self) -> Address;

    /// Returns the base address of the parent process.
    ///
    /// # Remarks
    ///
    /// This method is analog to the `OsProcessInfo::address` function.
    fn parent_process(&self) -> Address;

    /// Returns the actual base address of this module.
    ///
    /// # Remarks
    ///
    /// The base address is contained in the virtual address range of the process
    /// this module belongs to.
    fn base(&self) -> Address;

    /// Returns the size of the module.
    fn size(&self) -> usize;

    /// Returns the full name of the module.
    fn name(&self) -> String;
}

// TODO: Exports / Sections / etc
