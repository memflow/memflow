use clap::ArgMatches;
use serde_derive::Deserialize;

use flow_core::*;

#[cfg(feature = "connector-bridge")]
use flow_bridge::*;

#[derive(Clone, Deserialize)]
pub struct MachineConfig {
    pub name: Option<String>,
    pub url: Option<String>,
}

#[derive(Clone, Deserialize)]
pub struct Config {
    pub machine: Option<Vec<MachineConfig>>,
}

#[cfg(feature = "connector-bridge")]
pub fn try_parse(file_name: &str) -> Result<Config> {
    let cfg = std::fs::read_to_string(file_name)?;
    Ok(toml::from_str::<Config>(&cfg).map_err(Error::new)?)
}

#[cfg(feature = "connector-bridge")]
pub fn init_bridge(argv: &ArgMatches) -> Result<BridgeClient> {
    let url = {
        if argv.is_present("url") {
            argv.value_of("url").unwrap().to_owned()
        } else {
            let machines = try_parse(argv.value_of("config").unwrap_or_else(|| "memflow.toml"))
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

#[cfg(not(feature = "connector-bridge"))]
pub fn init_bridge(_argv: &ArgMatches) -> Result<super::EmptyVirtualMemory> {
    Err(Error::new("connector bridge is not enabled"))
}
