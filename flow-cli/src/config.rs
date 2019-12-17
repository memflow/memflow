use std::io::{Error, ErrorKind, Result};
use std::fs;

use toml;

use serde_derive::Deserialize;

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

pub fn try_parse(name: &str) -> Result<Config> {
    let cfg = fs::read_to_string(name).map_err(|e| Error::new(ErrorKind::Other, e))?;
    Ok(toml::from_str::<Config>(&cfg)?)
}