/*!
Abstraction over a address on the target system.
*/

use super::{Length, Offset};

use std::default::Default;
use std::fmt;
use std::ops;

/**
This type represents a address on the target system.
It internally holds a `u64` value but can also be used
when working in 32-bit environments.

This type will not handle overflow for 32-bit or 64-bit addresses / lengths.
*/
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Address(u64);

/// Constructs an `Address` from a `i32` value.
impl From<i32> for Address {
    fn from(item: i32) -> Self {
        Self { 0: item as u64 }
    }
}

/// Constructs an `Address` from a `u32` value.
impl From<u32> for Address {
    fn from(item: u32) -> Self {
        Self { 0: u64::from(item) }
    }
}

/// Constructs an `Address` from a `u64` value.
impl From<u64> for Address {
    fn from(item: u64) -> Self {
        Self { 0: item }
    }
}

/// Constructs an `Address` from a `usize` value.
impl From<usize> for Address {
    fn from(item: usize) -> Self {
        Self { 0: item as u64 }
    }
}

/// Constructs an `Address` from a `Length`.
impl From<Length> for Address {
    fn from(item: Length) -> Self {
        Self { 0: item.as_u64() }
    }
}

impl Address {
    /// A address with the value of zero.
    pub const NULL: Address = Address { 0: 0 };

    /// A address with an invalid value.
    pub const INVALID: Address = Address { 0: !0 };

    /// Returns an address with a value of zero.
    pub const fn null() -> Self {
        Address::NULL
    }

    /// Checks wether the address is zero or not.
    pub const fn is_null(self) -> bool {
        self.0 == 0
    }

    /// Returns an address with a invalid value.
    pub const fn invalid() -> Self {
        Address::INVALID
    }

    /// Checks wether the address is valid or not.
    pub const fn is_valid(self) -> bool {
        self.0 != !0
    }

    /// Converts the address into a `u32` value.
    pub const fn as_u32(self) -> u32 {
        self.0 as u32
    }

    /// Converts the address into a `u64` value.
    pub const fn as_u64(self) -> u64 {
        self.0
    }

    /// Converts the address into a `usize` value.
    pub const fn as_usize(self) -> usize {
        self.0 as usize
    }

    /// Aligns the containing address to the given page size. It returns the base address of the containing page.
    pub const fn as_page_aligned(self, page_size: Length) -> Address {
        Address {
            0: self.0 - self.0 % page_size.as_u64(),
        }
    }
}

/// Returns a address with a value of zero.
impl Default for Address {
    fn default() -> Self {
        Self::null()
    }
}

/// Adds a `Length` to a `Address` which results in a `Address`.
impl ops::Add<Length> for Address {
    type Output = Self;

    fn add(self, other: Length) -> Self {
        Self {
            0: self.0 + other.as_u64(),
        }
    }
}

/// Adds a `Length` to a `Address`.
impl ops::AddAssign<Length> for Address {
    fn add_assign(&mut self, other: Length) {
        *self = Self {
            0: self.0 + other.as_u64(),
        }
    }
}

// TODO: guarantee no underlfow
/// Subtracts a `Address` from a `Address` resulting in a `Length`.
impl ops::Sub for Address {
    type Output = Length;

    fn sub(self, other: Self) -> Length {
        Length::from(self.0 - other.0)
    }
}

// TODO: guarantee no underlfow
/// Subtracts a `Length` from a `Address` resulting in a `Address`.
impl ops::Sub<Length> for Address {
    type Output = Address;

    fn sub(self, other: Length) -> Address {
        Address::from(self.0 - other.as_u64())
    }
}

/// Subtracts a `Length` from a `Address`.
impl ops::SubAssign<Length> for Address {
    fn sub_assign(&mut self, other: Length) {
        *self = Self {
            0: self.0 - other.as_u64(),
        }
    }
}

/// Adds a `Offset` to a `Address` resulting in a `Address`.
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

/// Subtract a `Offset` from a `Address`.
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
