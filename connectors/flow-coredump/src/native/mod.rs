pub mod x64;
pub mod x86;

pub use x64::try_parse_coredump64;
pub use x86::try_parse_coredump32;

pub const DUMP_SIGNATURE: u32 = 0x45474150;
pub const DUMP_TYPE_FULL: u32 = 1;

pub const PHYSICAL_MEMORY_MAX_RUNS: usize = 0x20;
