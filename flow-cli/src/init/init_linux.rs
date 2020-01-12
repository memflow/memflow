use clap::ArgMatches;
use flow_core::Result;

pub fn init_procfs_connector(_argv: &ArgMatches) -> Result<flow_qemu_procfs::Memory> {
    flow_qemu_procfs::Memory::new()
}
