/*!
OS layers inventory interface.
*/

/// Exported memflow connector version
pub const MEMFLOW_OS_VERSION: i32 = 1;

/// Describes a connector
#[repr(C)]
pub struct OSDescriptor {
    /// The OS inventory api version for when the connector was built.
    /// This has to be set to `MEMFLOW_OS_VERSION` of memflow.
    ///
    /// If the versions mismatch the inventory will refuse to load.
    pub os_version: i32,

    /// The name of the connector.
    /// This name will be used when loading a OS from a connector inventory.
    pub name: &'static str,

    /// The vtable for all opaque function calls to the connector.
    pub vtable: OSFunctionTable,
}


