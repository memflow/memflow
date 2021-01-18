use crate::prelude::v1::*;

#[repr(C)]
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
pub struct ModuleInfo {
    /// Returns the address of the module header.
    ///
    /// # Remarks
    ///
    /// On Windows this will be the address where the [`PEB`](https://docs.microsoft.com/en-us/windows/win32/api/winternl/ns-winternl-peb) entry is stored.
    pub address: Address,
    /// The base address of the parent process.
    ///
    /// # Remarks
    ///
    /// This field is analog to the `ProcessInfo::address` field.
    pub parent_process: Address,
    /// The actual base address of this module.
    ///
    /// # Remarks
    ///
    /// The base address is contained in the virtual address range of the process
    /// this module belongs to.
    pub base: Address,
    /// Size of the module
    pub size: usize,
    /// Name of the module
    pub name: ReprCStr,
    /// Path of the module
    pub path: ReprCStr,
    /// Architecture of the module
    ///
    /// # Remarks
    ///
    /// Emulated processes often have 2 separate lists of modules, one visible to the emulated
    /// context (e.g. all 32-bit modules in a WoW64 process), and the other for all native modules
    /// needed to support the process emulation. This should be equal to either
    /// `ProcessInfo::proc_arch`, or `ProcessInfo::sys_arch` of the parent process.
    pub arch: ArchitectureIdent,
}

pub type ModuleInfoCallback<'a> = OpaqueCallback<'a, ModuleInfo>;

#[repr(C)]
#[derive(Clone, Debug)]
pub struct ModuleAddressInfo {
    pub address: Address,
    pub arch: ArchitectureIdent,
}

pub type ModuleAddressCallback<'a> = OpaqueCallback<'a, ModuleAddressInfo>;

// TODO: Exports / Sections / etc
