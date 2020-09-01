/*!
Module containing basic connector and inventory related functions.

This module provides basic building blocks when building connectors.
It contains a file i/o and memory mapped file interface
as well as a interface for interfacing with buffers.

This module also contains functions to interface with dynamically loaded connectors.
The inventory system is feature gated behind the `inventory` feature.
*/

pub mod args;
#[doc(hidden)]
pub use args::ConnectorArgs;

#[cfg(feature = "inventory")]
pub mod inventory;
#[doc(hidden)]
#[cfg(feature = "inventory")]
pub use inventory::{
    Connector, ConnectorDescriptor, ConnectorInstance, ConnectorInventory, ConnectorType,
    MEMFLOW_CONNECTOR_VERSION,
};

#[cfg(feature = "std")]
pub mod fileio;
#[doc(hidden)]
#[cfg(feature = "std")]
pub use fileio::FileIOMemory;

#[cfg(feature = "filemap")]
pub mod filemap;
#[cfg(feature = "filemap")]
pub use filemap::{
    MMAPInfo, MMAPInfoMut, ReadMappedFilePhysicalMemory, WriteMappedFilePhysicalMemory,
};

pub mod mmap;
#[doc(hidden)]
pub use mmap::MappedPhysicalMemory;
