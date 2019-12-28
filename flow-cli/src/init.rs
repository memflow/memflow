use clap::ArgMatches;
use flow_core::Result;

#[cfg(target_os = "linux")]
pub mod init_linux;

#[cfg(target_os = "linux")]
use flow_core::connector::qemu_procfs;

#[cfg(target_os = "linux")]
pub fn init_connector(argv: &ArgMatches) -> Result<qemu_procfs::Memory> {
    init_linux::init_procfs_connector(argv)
}

#[cfg(not(target_os = "linux"))]
pub mod init_other;

#[cfg(not(target_os = "linux"))]
use flow_core::connector::bridge::client::BridgeClient;

#[cfg(not(target_os = "linux"))]
pub fn init_connector(argv: &ArgMatches) -> Result<Bridgeclient> {
    init_other::init_bridge_connector(argv)
}
