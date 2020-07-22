use crate::error::Result;
use crate::mem::PhysicalMemory;

pub const MEMFLOW_PLUGIN_VERSION: i32 = 1;

pub struct ConnectorPlugin {
    pub memflow_plugin_version: i32,
    pub name: &'static str,
    pub factory: extern "C" fn(args: &str) -> Result<Box<dyn PhysicalMemory>>,
}

// TODO: handle plugin loading
