pub mod stdio;
pub use stdio::IOPhysicalMemory;

pub mod mmap;
pub use mmap::{MappedPhysicalMemory, ReadMappedFilePhysicalMemory, WriteMappedFilePhysicalMemory};
