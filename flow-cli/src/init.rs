use clap::ArgMatches;

use crate::config;

use flow_core::mem::*;
use flow_core::{Error, Result};

// TODO: feature
#[cfg(any(linux))]
use flow_core::connector::qemu_procfs;

use flow_core::connector::bridge::client::BridgeClient;

/*
pub enum Connector {
    Bridge(BridgeClient),
    QemuProcFS(qemu_procfs::Memory),
}

pub fn init_connector(argv: &ArgMatches) -> Result<Connector> {
    match argv.value_of("connector").unwrap_or_else(|| "qemu_procfs") {
        "bridge" => Ok(Connector::Bridge(init_bridge_connector(argv)?)),
        "qemu_procfs" => Ok(Connector::QemuProcFS(qemu_procfs::Memory::new()?)),
        _ => panic!("invalid connector type"), // TODO: can clap restrict this?
    }
}
*/

// TODO: feature
#[cfg(any(linux))]
pub fn init_bridge_connector(argv: &ArgMatches) -> Result<qemu_procfs::Memory> {
    qemu_procfs::Memory::new().unwrap()
}

// TODO: feature
#[cfg(not(any(linux)))]
pub fn init_bridge_connector(argv: &ArgMatches) -> Result<BridgeClient> {
    let url = {
        if argv.is_present("url") {
            argv.value_of("url").unwrap().to_owned()
        } else {
            let machines =
                config::try_parse(argv.value_of("config").unwrap_or_else(|| "memflow.toml"))
                    .unwrap()
                    .machine
                    .unwrap(); // TODO: proper error handling / feedback

            let machine = {
                if argv.is_present("machine") {
                    machines
                        .iter()
                        .filter(|m| m.name.as_ref().unwrap() == argv.value_of("machine").unwrap())
                        .nth(0)
                        .ok_or_else(|| {
                            std::io::Error::new(std::io::ErrorKind::Other, "machine not found")
                        })
                } else if machines.len() == 1 {
                    Ok(&machines[0])
                } else {
                    Err(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        "no machine specified",
                    ))
                }
            }
            .unwrap();

            machine.url.to_owned().unwrap()
        }
    };

    BridgeClient::connect(url.as_str())
}
