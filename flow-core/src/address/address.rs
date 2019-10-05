use std::ops;
use std::fmt;

#[derive(Debug, PartialEq, PartialOrd)]
pub struct Address {
    pub addr: u64,
}

impl Address {
    pub fn valid(&self) -> bool {
        self.addr != 0
    }
}

impl From<u64> for Address {
    fn from(item: u64) -> Self {
        Self{ addr: item, }
    }
}

impl ops::Add for Address {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self{ addr: self.addr + other.addr, }
    }
}

impl ops::AddAssign for Address {
    fn add_assign(&mut self, other: Self) {
        *self = Self{ addr: self.addr + other.addr, }
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

impl fmt::LowerHex for Address {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:x}", self.addr)
    }
}
