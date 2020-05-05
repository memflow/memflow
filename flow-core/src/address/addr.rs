use std::default::Default;
use std::fmt;
use std::ops;

use super::{Length, Offset};

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Address(u64);

impl From<i32> for Address {
    fn from(item: i32) -> Self {
        Self { 0: item as u64 }
    }
}

impl From<u32> for Address {
    fn from(item: u32) -> Self {
        Self { 0: u64::from(item) }
    }
}

impl From<u64> for Address {
    fn from(item: u64) -> Self {
        Self { 0: item }
    }
}

impl From<usize> for Address {
    fn from(item: usize) -> Self {
        Self { 0: item as u64 }
    }
}

impl From<Length> for Address {
    fn from(item: Length) -> Self {
        Self { 0: item.as_u64() }
    }
}

impl Address {
    pub const NULL: Address = Address { 0: 0 };

    pub const fn null() -> Self {
        Address { 0: 0 }
    }

    pub const fn is_null(self) -> bool {
        self.0 == 0
    }

    pub fn as_len(self) -> Length {
        Length::from(self.0)
    }

    pub const fn as_u32(self) -> u32 {
        self.0 as u32
    }

    pub const fn as_u64(self) -> u64 {
        self.0
    }

    pub const fn as_usize(self) -> usize {
        self.0 as usize
    }

    pub const fn as_page_aligned(self, page_size: Length) -> Address {
        Address {
            0: self.0 - self.0 % page_size.as_u64(),
        }
    }
}

impl Default for Address {
    fn default() -> Self {
        Self::null()
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

// Address + Offset => Address
#[allow(clippy::suspicious_op_assign_impl, clippy::suspicious_arithmetic_impl)]
impl ops::Add<Offset> for Address {
    type Output = Address;

    fn add(self, other: Offset) -> Address {
        Address::from(if other.as_i64() >= 0 {
            self.0 + other.as_i64() as u64
        } else {
            self.0 - other.as_i64().abs() as u64
        })
    }
}

// Address -= Offset
#[allow(clippy::suspicious_op_assign_impl, clippy::suspicious_arithmetic_impl)]
impl ops::AddAssign<Offset> for Address {
    fn add_assign(&mut self, other: Offset) {
        *self = Self {
            0: if other.as_i64() >= 0 {
                self.0 + other.as_i64() as u64
            } else {
                self.0 - other.as_i64().abs() as u64
            },
        }
    }
}

impl fmt::Debug for Address {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:x}", self.0)
    }
}
impl fmt::UpperHex for Address {
    #[inline(always)]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:X}", self.0)
    }
}
impl fmt::LowerHex for Address {
    #[inline(always)]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:x}", self.0)
    }
}
impl fmt::Display for Address {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:x}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
            Address::from(0xFFF1_2345u64).as_page_aligned(Length::from_b(0x10000)),
            Address::from(0xFFF1_0000u64)
        );
    }

    #[test]
    fn test_ops() {
        assert_eq!(Address::from(10) + Length::from(5), Address::from(15));

        assert_eq!(Address::from(10) - Address::from(5), Length::from(5));
        assert_eq!(Address::from(100) - Length::from(5), Address::from(95));
    }

    #[test]
    fn test_offset() {
        assert_eq!(Address::from(10) + Offset::from(5), Address::from(15));
        assert_eq!(Address::from(10) + Offset::from(-5), Address::from(5));
    }
}
