pub mod log;

pub mod types;

pub mod connectors;

pub mod mem;

pub mod architecture;

pub mod util;

#[no_mangle]
pub extern "C" fn gone(_arch: &memflow::architecture::ArchitectureIdent) {}
