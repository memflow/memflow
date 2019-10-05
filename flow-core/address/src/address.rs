use std::ops;
use std::fmt;

use crate::length::Length;

#[derive(Debug, Copy, Clone, PartialEq, PartialOrd)]
pub struct Address {
    pub addr: u64,
}

impl fmt::LowerHex for Address {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:x}", self.addr)
    }
}

impl From<u64> for Address {
    fn from(item: u64) -> Self {
        Self{ addr: item, }
    }
}

impl Address {
    pub fn is_null(&self) -> bool {
        self.addr == 0
    }
}

impl ops::Add for Address {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self{ addr: self.addr + other.addr, }
    }
}

impl ops::Add<Length> for Address {
    type Output = Self;

    fn add(self, other: Length) -> Self {
        Self{ addr: self.addr + other.as_u64(), }
    }
}

impl ops::AddAssign for Address {
    fn add_assign(&mut self, other: Self) {
        *self = Self{ addr: self.addr + other.addr, }
    }
}

impl ops::AddAssign<Length> for Address {
    fn add_assign(&mut self, other: Length) {
        *self = Self{ addr: self.addr + other.as_u64(), }
    }
}

impl ops::Sub for Address {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self{ addr: self.addr - other.addr, }
    }
}

impl ops::SubAssign for Address {
    fn sub_assign(&mut self, other: Self) {
        *self = Self{ addr: self.addr - other.addr, }
    }
}
