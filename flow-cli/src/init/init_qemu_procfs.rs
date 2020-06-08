use clap::ArgMatches;

use flow_core::*;

#[cfg(all(target_os = "linux", feature = "connector-qemu-procfs"))]
pub fn init_qemu_procfs(argv: &ArgMatches) -> Result<flow_qemu_procfs::Memory> {
    if argv.is_present("connector_args") {
        flow_qemu_procfs::Memory::with_name(argv.value_of("connector_args").unwrap())
    } else {
        flow_qemu_procfs::Memory::new()
    }
}

#[cfg(all(feature = "connector-qemu-procfs", not(target_os = "linux")))]
pub fn init_qemu_procfs(argv: &ArgMatches) -> Result<super::EmptyVirtualMemory> {
    Err(Error::new(
        "connector qemu_procfs is not available on this system",
    ))
}

#[cfg(not(feature = "connector-qemu-procfs"))]
pub fn init_qemu_procfs(argv: &ArgMatches) -> Result<super::EmptyVirtualMemory> {
    Err(Error::new("connector qemu-procfs is not enabled"))
}
