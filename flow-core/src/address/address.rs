use std::fmt;
use std::ops;

use crate::address::Length;

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Address(u64);

impl fmt::LowerHex for Address {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:x}", self.0)
    }
}

impl From<i32> for Address {
    fn from(item: i32) -> Self {
        Self { 0: item as u64 }
    }
}

impl From<u32> for Address {
    fn from(item: u32) -> Self {
        Self { 0: item as u64 }
    }
}

impl From<u64> for Address {
    fn from(item: u64) -> Self {
        Self { 0: item }
    }
}

impl Address {
    pub fn null() -> Self {
        Address { 0: 0 }
    }

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
        Address {
            0: self.0 & (!(page_size.as_u64() - 1)),
        }
    }
}

// Address + Length => Address
impl ops::Add<Length> for Address {
    type Output = Self;

    fn add(self, other: Length) -> Self {
        Self {
            0: self.0 + other.as_u64(),
        }
    }
}

// Address += Length
impl ops::AddAssign<Length> for Address {
    fn add_assign(&mut self, other: Length) {
        *self = Self {
            0: self.0 + other.as_u64(),
        }
    }
}

// TODO: guarantee no underlfow
// Address - Address => Length
impl ops::Sub for Address {
    type Output = Length;

    fn sub(self, other: Self) -> Length {
        Length::from(self.0 - other.0)
    }
}

// TODO: guarantee no underlfow
// Address - Length => Address
impl ops::Sub<Length> for Address {
    type Output = Address;

    fn sub(self, other: Length) -> Address {
        Address::from(self.0 - other.as_u64())
    }
}

// Address -= Length
impl ops::SubAssign<Length> for Address {
    fn sub_assign(&mut self, other: Length) {
        *self = Self {
            0: self.0 - other.as_u64(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::address::Address;
    use crate::address::Length;

    #[test]
    fn test_from() {
        assert_eq!(Address::null().is_null(), true);
        assert_eq!(Address::from(1337).as_u64(), 1337);
        assert_eq!(Address::from(4321).as_usize(), 4321);
    }

    #[test]
    fn test_alignment() {
        assert_eq!(
            Address::from(0x1234).as_page_aligned(Length::from_kb(4)),
            Address::from(0x1000)
        );
        assert_eq!(
            Address::from(0xFFF12345).as_page_aligned(Length::from_b(0x10000)),
            Address::from(0xFFF10000)
        );
    }

    #[test]
    fn test_ops() {
        assert_eq!(Address::from(10) + Length::from(5), Address::from(15));

        assert_eq!(Address::from(10) - Address::from(5), Length::from(5));
        assert_eq!(Address::from(100) - Length::from(5), Address::from(95));
    }
}
