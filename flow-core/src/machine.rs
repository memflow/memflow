use std::io::Result;

use crate::cpu::{Architecture, CPU};
use crate::mem::PhysicalMemory;
use crate::os::OperatingSystem;

// Machine has a cpu+mem+net implementation
pub struct Machine <'a> {
    pub cpu: CPU,
    pub mem: &'a mut dyn PhysicalMemory,
    pub os: Option<OperatingSystem>,
}

impl Machine<'_> {
    pub fn new(mem: &mut dyn PhysicalMemory) -> Machine {
        Machine{
            cpu: CPU{
                arch: None,
            },
            mem: mem,
            os: None,
        }
    }
}
