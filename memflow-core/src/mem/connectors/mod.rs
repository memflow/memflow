#[cfg(feature = "plugins")]
pub mod plugin;
#[cfg(feature = "plugins")]
pub use plugin::{ConnectorDescriptor, ConnectorInventory, MEMFLOW_CONNECTOR_VERSION};

pub mod fileio;
pub use fileio::IOPhysicalMemory;

pub mod mmap;
pub use mmap::{MappedPhysicalMemory, ReadMappedFilePhysicalMemory, WriteMappedFilePhysicalMemory};
