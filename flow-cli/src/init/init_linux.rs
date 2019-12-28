use clap::ArgMatches;
use flow_core::Result;

use flow_core::connector::qemu_procfs;

pub fn init_procfs_connector(_argv: &ArgMatches) -> Result<qemu_procfs::Memory> {
    qemu_procfs::Memory::new()
}
