use std::ops;
use std::fmt;

#[derive(Debug, Copy, Clone, PartialEq, PartialOrd)]
pub struct Length(u64);

impl fmt::LowerHex for Length {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:x}", self.0)
    }
}

impl From<i32> for Length {
    fn from(item: i32) -> Self {
        Self{ 0: item as u64, }
    }
}

impl From<u64> for Length {
    fn from(item: u64) -> Self {
        Self{ 0: item, }
    }
}

impl From<usize> for Length {
    fn from(item: usize) -> Self {
        Self{ 0: item as u64, }
    }
}

impl Length {
    pub fn zero() -> Self {
        Length::from(0)
    }

    pub fn as_u64(&self) -> u64 {
        self.0
    }

    pub fn as_usize(&self) -> usize {
        self.0 as usize
    }

    pub fn from_b(len: u64) -> Self {
        Length{ 0: len, }
    }

    pub fn from_kb(len: u64) -> Self {
        Length{ 0: len * 1024, }
    }

    pub fn from_kib(len: u64) -> Self {
        Length{ 0: len * 1024 * 8, }
    }

    pub fn from_mb(len: u64) -> Self {
        Length{ 0: len * 1024 * 1024, }
    }

    pub fn from_mib(len: u64) -> Self {
        Length{ 0: len * 1024 * 1024 * 8, }
    }

    pub fn from_gb(len: u64) -> Self {
        Length{ 0: len * 1024 * 1024 * 1024, }
    }

    pub fn from_gib(len: u64) -> Self {
        Length{ 0: len * 1024 * 1024 * 1024 * 8, }
    }
}

impl ops::Add for Length {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self{ 0: self.0 + other.0, }
    }
}

impl ops::Add<i32> for Length {
    type Output = Self;

    fn add(self, other: i32) -> Self {
        Self{ 0: self.0 + (other as u64), }
    }
}

impl ops::Add<u32> for Length {
    type Output = Self;

    fn add(self, other: u32) -> Self {
        Self{ 0: self.0 + (other as u64), }
    }
}

impl ops::Add<i64> for Length {
    type Output = Self;

    fn add(self, other: i64) -> Self {
        Self{ 0: self.0 + (other as u64), }
    }
}

impl ops::Add<u64> for Length {
    type Output = Self;

    fn add(self, other: u64) -> Self {
        Self{ 0: self.0 + other, }
    }
}

impl ops::AddAssign for Length {
    fn add_assign(&mut self, other: Self) {
        *self = Self{ 0: self.0 + other.0, }
    }
}

impl ops::AddAssign<i32> for Length {
    fn add_assign(&mut self, other: i32) {
        *self = Self{ 0: self.0 + (other as u64), }
    }
}

impl ops::AddAssign<u32> for Length {
    fn add_assign(&mut self, other: u32) {
        *self = Self{ 0: self.0 + (other as u64), }
    }
}

impl ops::AddAssign<i64> for Length {
    fn add_assign(&mut self, other: i64) {
        *self = Self{ 0: self.0 + (other as u64), }
    }
}

impl ops::AddAssign<u64> for Length {
    fn add_assign(&mut self, other: u64) {
        *self = Self{ 0: self.0 + other, }
    }
}

impl ops::Sub for Length {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self{ 0: self.0 - other.0, }
    }
}

impl ops::SubAssign for Length {
    fn sub_assign(&mut self, other: Self) {
        *self = Self{ 0: self.0 - other.0, }
    }
}
