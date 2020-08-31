pub mod ntos;
pub mod start_block;
pub mod sysproc;

use std::prelude::v1::*;

pub use start_block::StartBlock;

use std::cmp::{Ord, Ordering, PartialEq};
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

#[derive(Debug, Clone, Copy, Default)]
#[cfg_attr(feature = "serde", derive(::serde::Serialize))]
#[repr(C)]
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

    pub fn mask_build_number(mut self) -> Self {
        self.nt_build_number &= 0xFFFF;
        self
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

    pub fn as_tuple(&self) -> (u32, u32, u32) {
        (
            self.major_version(),
            self.minor_version(),
            self.build_number(),
        )
    }
}

impl PartialOrd for Win32Version {
    fn partial_cmp(&self, other: &Win32Version) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Win32Version {
    fn cmp(&self, other: &Win32Version) -> Ordering {
        if self.nt_build_number != 0 && other.nt_build_number != 0 {
            return self.nt_build_number.cmp(&other.nt_build_number);
        }

        if self.nt_major_version != other.nt_major_version {
            self.nt_major_version.cmp(&other.nt_major_version)
        } else if self.nt_minor_version != other.nt_minor_version {
            self.nt_minor_version.cmp(&other.nt_minor_version)
        } else {
            Ordering::Equal
        }
    }
}

impl PartialEq for Win32Version {
    fn eq(&self, other: &Win32Version) -> bool {
        if self.nt_build_number != 0 && other.nt_build_number != 0 {
            self.nt_build_number.eq(&other.nt_build_number)
        } else {
            self.nt_major_version == other.nt_major_version
                && self.nt_minor_version == other.nt_minor_version
        }
    }
}

impl Eq for Win32Version {}

impl From<(u32, u32)> for Win32Version {
    fn from((nt_major_version, nt_minor_version): (u32, u32)) -> Win32Version {
        Win32Version {
            nt_major_version,
            nt_minor_version,
            nt_build_number: 0,
        }
    }
}

impl From<(u32, u32, u32)> for Win32Version {
    fn from(
        (nt_major_version, nt_minor_version, nt_build_number): (u32, u32, u32),
    ) -> Win32Version {
        Win32Version {
            nt_major_version,
            nt_minor_version,
            nt_build_number,
        }
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
