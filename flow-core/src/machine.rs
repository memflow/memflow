/*use std::io::Result;

use crate::arch::Architecture;
use crate::mem::{PhysicalRead, VirtualRead, PhysicalWrite, VirtualWrite};
use crate::os::OperatingSystem;
*/

// Machine has a cpu+mem+net implementation
/*
pub struct Machine<T: PhysicalRead + VirtualRead, U: PhysicalWrite + VirtualWrite> {
    pub arch: Option<Architecture>,
    // cpu trait
    pub mem_read: Option<T>,
    pub mem_write: Option<U>,
    pub os: Option<OperatingSystem>,
}

impl Machine {
    pub fn new() -> Machine {
        Machine{
            arch: None,
            mem_read: None,
            mem_write: None,
            os: None,
        }
    }
}
*/
