pub mod plugin;
pub use plugin::{ConnectorPlugin, MEMFLOW_PLUGIN_VERSION};

pub mod fileio;
pub use fileio::IOPhysicalMemory;

pub mod mmap;
pub use mmap::{MappedPhysicalMemory, ReadMappedFilePhysicalMemory, WriteMappedFilePhysicalMemory};
