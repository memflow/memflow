use std::ops;
use std::fmt;

use crate::length::Length;

#[derive(Debug, Copy, Clone, PartialEq, PartialOrd)]
pub struct Address(u64);

impl fmt::LowerHex for Address {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:x}", self.0)
    }
}

impl From<u64> for Address {
    fn from(item: u64) -> Self {
        Self{ 0: item, }
    }
}

impl Address {
    pub fn is_null(&self) -> bool {
        self.0 == 0
    }

    pub fn as_u64(&self) -> u64 {
        self.0
    }

    pub fn as_usize(&self) -> usize {
        self.0 as usize
    }

    pub fn as_page_aligned(&self, page_size: Length) -> Address {
        Address{ 0: self.0 & (!(page_size.as_u64() - 1)) }
    }
}

impl ops::Add for Address {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self{ 0: self.0 + other.0, }
    }
}

impl ops::Add<Length> for Address {
    type Output = Self;

    fn add(self, other: Length) -> Self {
        Self{ 0: self.0 + other.as_u64(), }
    }
}

impl ops::AddAssign for Address {
    fn add_assign(&mut self, other: Self) {
        *self = Self{ 0: self.0 + other.0, }
    }
}

impl ops::AddAssign<Length> for Address {
    fn add_assign(&mut self, other: Length) {
        *self = Self{ 0: self.0 + other.as_u64(), }
    }
}

// Address - Address => Length
impl ops::Sub for Address {
    type Output = Length;

    fn sub(self, other: Self) -> Length {
        Length::from(self.0 - other.0)
    }
}
