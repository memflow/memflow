use std::default::Default;
use std::fmt;
use std::ops;

use super::{Address, Length};

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Offset(i64);

impl fmt::LowerHex for Offset {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:x}", self.0)
    }
}

impl From<i64> for Offset {
    fn from(item: i64) -> Self {
        Self { 0: item as i64 }
    }
}

impl From<i32> for Offset {
    fn from(item: i32) -> Self {
        Self { 0: item as i64 }
    }
}

impl From<i16> for Offset {
    fn from(item: i16) -> Self {
        Self { 0: item as i64 }
    }
}

impl Offset {
    pub fn zero() -> Self {
        Offset::from(0)
    }

    pub fn is_zero(self) -> bool {
        self.0 == 0
    }

    pub fn as_i64(self) -> i64 {
        self.0
    }

    pub fn as_i32(self) -> i32 {
        self.0 as i32
    }
}

impl Default for Offset {
    fn default() -> Self {
        Self::zero()
    }
}

// TODO: add overwrites
