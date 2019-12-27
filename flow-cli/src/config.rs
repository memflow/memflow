use std::cell::RefCell;
use std::fs;
use std::rc::Rc;

use toml;

use serde_derive::Deserialize;

use flow_core::connector::bridge::client::BridgeClient;
use flow_core::mem::*;
use flow_core::{Error, Result};
use flow_win32::win::Windows;

#[derive(Clone, Deserialize)]
pub struct MachineConfig {
    pub name: Option<String>,
    pub url: Option<String>,
    pub os: Option<String>,
}

#[derive(Clone, Deserialize)]
pub struct Config {
    pub machine: Option<Vec<MachineConfig>>,
}

pub fn try_parse(file_name: &str) -> Result<Config> {
    let cfg = fs::read_to_string(file_name)?;
    Ok(toml::from_str::<Config>(&cfg).map_err(Error::new)?)
}
/*
pub enum OS<T: VirtualRead> {
    Win32(Windows<T>),
}

pub struct Machine<T: VirtualRead> {
    mem: Rc<RefCell<T>>,
    os: OS<T>,
}

// TODO: dedicated os traits
// TODO: forward declares into crate root
pub fn try_init_machine<T>(file_name: &str, machine_name: &str) -> Result<Machine<T>>
where
    T: VirtualRead,
{
    let machines = try_parse(file_name)?.machine.ok_or_else(|| Error::new("machine not found"))?;

    let machine = machines
        .iter()
        .filter(|m| m.name.as_ref().unwrap() == machine_name)
        .nth(0)
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::Other, "machine not found"))?;

    // TODO: handle all types of connectors in config
    let bridge = match BridgeClient::connect(
        machine.url.to_owned().unwrap().as_str(), /* TODO: error handling */
    ) {
        Ok(br) => br,
        Err(e) => {
            println!("couldn't connect to bridge: {:?}", e);
            return Err(e);
        }
    };

    // TODO: diff os
    let bridgerc = Rc::new(RefCell::new(bridge));
    let os = match machine
        .os
        .to_owned()
        .unwrap_or_else(|| String::from("win32"))
        .as_str()
    {
        "win32" => Ok(OS::Win32(flow_win32::init(bridgerc).map_err(Error::new)?)),
        //"linux" => {},
        _ => Err(Error::new("invalid os")),
    }?;

    Ok(Machine{
        mem: bridgerc,
        os,
    })
}

//pub fn try_init(file_name: &str) {}
*/
