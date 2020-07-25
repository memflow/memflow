pub mod x64;
pub mod x86;

pub mod bitmap_dump;
pub mod full_memory_dump;

pub use x64::parse_coredump64;
pub use x86::parse_coredump32;

/// Coredump Header Signature
pub const DUMP_SIGNATURE: u32 = 0x45474150;

/// The type of the Coredump
mod dump_type {
    pub const FULL: u32 = 1;
    pub const BIT_MAP: u32 = 5;
}

/// The number of PhysicalMemoryRuns contained in the Header
pub const PHYSICAL_MEMORY_MAX_RUNS: usize = 0x20;
