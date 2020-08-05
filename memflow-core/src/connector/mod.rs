/*!
Module containing basic connector and plugin related functions.

This module provides basic building blocks when building connectors.
It contains a file i/o and memory mapped file interface
as well as a interface for interfacing with buffers.

This module also contains functions to interface with plugins.
The plugins system is feature gated behind the `plugins` feature.
*/

pub mod args;
#[doc(hidden)]
pub use args::ConnectorArgs;

#[cfg(feature = "plugins")]
pub mod plugin;
#[doc(hidden)]
#[cfg(feature = "plugins")]
pub use plugin::{
    Connector, ConnectorDescriptor, ConnectorInventory, PluginConnector, MEMFLOW_CONNECTOR_VERSION,
};

pub mod fileio;
#[doc(hidden)]
pub use fileio::FileIOMemory;

pub mod mmap;
#[doc(hidden)]
pub use mmap::{MappedPhysicalMemory, ReadMappedFilePhysicalMemory, WriteMappedFilePhysicalMemory};
