pub mod ntos;
pub mod start_block;
pub mod sysproc;

use std::prelude::v1::*;

pub use start_block::StartBlock;

use std::fmt;

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(::serde::Serialize))]
pub struct Win32GUID {
    pub file_name: String,
    pub guid: String,
}

impl Win32GUID {
    pub fn new(file_name: &str, guid: &str) -> Self {
        Self {
            file_name: file_name.to_string(),
            guid: guid.to_string(),
        }
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(::serde::Serialize))]
pub struct Win32Version {
    nt_major_version: u32,
    nt_minor_version: u32,
    nt_build_number: u32,
}

impl Win32Version {
    pub fn new(nt_major_version: u32, nt_minor_version: u32, nt_build_number: u32) -> Self {
        Self {
            nt_major_version,
            nt_minor_version,
            nt_build_number,
        }
    }

    pub fn major_version(&self) -> u32 {
        self.nt_major_version
    }

    pub fn minor_version(&self) -> u32 {
        self.nt_minor_version
    }

    pub fn build_number(&self) -> u32 {
        self.nt_build_number & 0xFFFF
    }

    pub fn is_checked_build(&self) -> bool {
        (self.nt_build_number & 0xF0000000) == 0xC0000000
    }
}

impl fmt::Display for Win32Version {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.nt_major_version != 0 {
            write!(
                f,
                "{}.{}.{}",
                self.major_version(),
                self.minor_version(),
                self.build_number()
            )
        } else {
            write!(f, "{}", self.build_number())
        }
    }
}
