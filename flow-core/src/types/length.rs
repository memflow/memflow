/*!
Abstraction over a length.
This is usually being used in conjunction with [`Address`](../address/index.html)
*/

use std::default::Default;
use std::fmt;
use std::ops;

/**
This type represents a length.
It internally holds a `u64` value but can also be used
when working in 32-bit environments.

This type will not handle overflow for 32-bit or 64-bit addresses / lengths.
*/
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Length(u64);

/// Constructs a `Length` from a `u16` value.
impl From<u16> for Length {
    fn from(item: u16) -> Self {
        Self { 0: u64::from(item) }
    }
}

/// Constructs a `Length` from a `i16` value.
impl From<i16> for Length {
    fn from(item: i16) -> Self {
        Self { 0: item as u64 }
    }
}

/// Constructs a `Length` from a `u32` value.
impl From<u32> for Length {
    fn from(item: u32) -> Self {
        Self { 0: u64::from(item) }
    }
}

/// Constructs a `Length` from a `i32` value.
impl From<i32> for Length {
    fn from(item: i32) -> Self {
        Self { 0: item as u64 }
    }
}

/// Constructs a `Length` from a `u64` value.
impl From<u64> for Length {
    fn from(item: u64) -> Self {
        Self { 0: item }
    }
}

/// Constructs a `Length` from a `i64` value.
impl From<i64> for Length {
    fn from(item: i64) -> Self {
        Self { 0: item as u64 }
    }
}

/// Constructs a `Length` from a `usize` value.
impl From<usize> for Length {
    fn from(item: usize) -> Self {
        Self { 0: item as u64 }
    }
}

impl Length {
    /// A length with the value of zero.
    pub const ZERO: Length = Length { 0: 0 };

    /// Returns a length with a value of zero.
    pub const fn zero() -> Self {
        Length { 0: 0 }
    }

    /// Checks wether the length is zero or not.
    pub const fn is_zero(self) -> bool {
        self.0 == 0
    }

    /// Converts the length into a `u32` value.
    pub const fn as_u32(self) -> u32 {
        self.0 as u32
    }

    /// Converts the length into a `u64` value.
    pub const fn as_u64(self) -> u64 {
        self.0
    }

    /// Converts the length into a `usize` value.
    pub const fn as_usize(self) -> usize {
        self.0 as usize
    }

    /// Constructs a length from the given number of bytes.
    pub const fn from_b(len: u64) -> Self {
        Length { 0: len }
    }

    /// Constructs a length from the given number of kilobytes.
    pub const fn from_kb(len: u64) -> Self {
        Length { 0: len * 1024 }
    }

    /// Constructs a length from the given number of kilobits.
    pub const fn from_kib(len: u64) -> Self {
        Length { 0: len * 1024 / 8 }
    }

    /// Constructs a length from the given number of megabytes.
    pub const fn from_mb(len: u64) -> Self {
        Length {
            0: len * 1024 * 1024,
        }
    }

    /// Constructs a length from the given number of megabits.
    pub const fn from_mib(len: u64) -> Self {
        Length {
            0: len * 1024 * 1024 / 8,
        }
    }

    /// Constructs a length from the given number of gigabytes.
    pub const fn from_gb(len: u64) -> Self {
        Length {
            0: len * 1024 * 1024 * 1024,
        }
    }

    /// Constructs a length from the given number of gigabits.
    pub const fn from_gib(len: u64) -> Self {
        Length {
            0: len * 1024 * 1024 * 1024 / 8,
        }
    }

    /// Constructs a length containing the size of an object.
    pub const fn size_of<T>() -> Self {
        Length {
            0: std::mem::size_of::<T>() as u64,
        }
    }
}

/// Returns a length with a value of zero.
impl Default for Length {
    fn default() -> Self {
        Self::zero()
    }
}

/// Adds a `Length` to a `Length` which results in a `Length`.
impl ops::Add for Length {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            0: self.0 + other.0,
        }
    }
}

/// Adds a `i32` value to a `Length`.
impl ops::Add<i32> for Length {
    type Output = Self;

    fn add(self, other: i32) -> Self {
        Self {
            0: self.0 + (other as u64),
        }
    }
}

/// Adds a `u32` value to a `Length` which results in a `Length`.
impl ops::Add<u32> for Length {
    type Output = Self;

    fn add(self, other: u32) -> Self {
        Self {
            0: self.0 + (u64::from(other)),
        }
    }
}

/// Adds a `i64` value to a `Length` which results in a `Length`.
impl ops::Add<i64> for Length {
    type Output = Self;

    fn add(self, other: i64) -> Self {
        Self {
            0: self.0 + (other as u64),
        }
    }
}

/// Adds a `u64` value to a `Length` which results in a `Length`.
impl ops::Add<u64> for Length {
    type Output = Self;

    fn add(self, other: u64) -> Self {
        Self { 0: self.0 + other }
    }
}

/// Adds a `usize` value to a `Length` which results in a `Length`.
impl ops::Add<usize> for Length {
    type Output = Self;

    fn add(self, other: usize) -> Self {
        Self {
            0: self.0 + (other as u64),
        }
    }
}

/// Multiplies a `usize` value with a `Length` which results in a `Length`.
impl ops::Mul<usize> for Length {
    type Output = Self;

    fn mul(self, other: usize) -> Self {
        Self {
            0: self.0 * (other as u64),
        }
    }
}

/// Adds a `Length` to a `Length`.
impl ops::AddAssign for Length {
    fn add_assign(&mut self, other: Self) {
        *self = Self {
            0: self.0 + other.0,
        }
    }
}

/// Adds a `i32` value to a `Length`.
impl ops::AddAssign<i32> for Length {
    fn add_assign(&mut self, other: i32) {
        *self = Self {
            0: self.0 + (other as u64),
        }
    }
}

/// Adds a `u32` value to a `Length`.
impl ops::AddAssign<u32> for Length {
    fn add_assign(&mut self, other: u32) {
        *self = Self {
            0: self.0 + (u64::from(other)),
        }
    }
}

/// Adds a `i64` value to a `Length`.
impl ops::AddAssign<i64> for Length {
    fn add_assign(&mut self, other: i64) {
        *self = Self {
            0: self.0 + (other as u64),
        }
    }
}

/// Adds a `u64` value to a `Length`.
impl ops::AddAssign<u64> for Length {
    fn add_assign(&mut self, other: u64) {
        *self = Self { 0: self.0 + other }
    }
}

/// Adds a `usize` value to a `Length`.
impl ops::AddAssign<usize> for Length {
    fn add_assign(&mut self, other: usize) {
        *self = Self {
            0: self.0 + (other as u64),
        }
    }
}

/// Subtracts a `Length` from a `Length` resulting in a `Length`.
impl ops::Sub for Length {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self {
            0: self.0 - other.0,
        }
    }
}

/// Subtracts a `i32` value from a `Length` resulting in a `Length`.
impl ops::Sub<i32> for Length {
    type Output = Self;

    fn sub(self, other: i32) -> Self {
        Self {
            0: self.0 - (other as u64),
        }
    }
}

/// Subtracts a `u32` value from a `Length` resulting in a `Length`.
impl ops::Sub<u32> for Length {
    type Output = Self;

    fn sub(self, other: u32) -> Self {
        Self {
            0: self.0 - (u64::from(other)),
        }
    }
}

/// Subtracts a `i64` value from a `Length` resulting in a `Length`.
impl ops::Sub<i64> for Length {
    type Output = Self;

    fn sub(self, other: i64) -> Self {
        Self {
            0: self.0 - (other as u64),
        }
    }
}

/// Subtracts a `u64` value from a `Length` resulting in a `Length`.
impl ops::Sub<u64> for Length {
    type Output = Self;

    fn sub(self, other: u64) -> Self {
        Self { 0: self.0 - other }
    }
}

/// Subtracts a `usize` value from a `Length` resulting in a `Length`.
impl ops::Sub<usize> for Length {
    type Output = Self;

    fn sub(self, other: usize) -> Self {
        Self {
            0: self.0 - (other as u64),
        }
    }
}

/// Subtracts a `Length` from a `Length`.
impl ops::SubAssign for Length {
    fn sub_assign(&mut self, other: Self) {
        *self = Self {
            0: self.0 - other.0,
        }
    }
}

impl fmt::Debug for Length {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:x}", self.0)
    }
}
impl fmt::UpperHex for Length {
    #[inline(always)]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:X}", self.0)
    }
}
impl fmt::LowerHex for Length {
    #[inline(always)]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:x}", self.0)
    }
}
impl fmt::Display for Length {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:x}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from() {
        assert_eq!(Length::zero().as_u64(), 0);
        assert_eq!(Length::from(1337).as_u64(), 1337);
        assert_eq!(Length::from(4321).as_usize(), 4321);
        assert_eq!(Length::from_b(500), Length::from(500));
        assert_eq!(Length::from_kb(20), Length::from(20 * 1024));
        assert_eq!(Length::from_kib(123), Length::from(123 * 1024 / 8));
        assert_eq!(Length::from_mb(20), Length::from(20 * 1024 * 1024));
        assert_eq!(Length::from_mib(52), Length::from(52 * 1024 * 1024 / 8));
        assert_eq!(
            Length::from_gb(20),
            Length::from(20u64 * 1024 * 1024 * 1024)
        );
        assert_eq!(
            Length::from_gib(52),
            Length::from(52u64 * 1024 * 1024 * 1024 / 8)
        );
    }

    #[test]
    fn test_ops() {
        assert_eq!(Length::from(100) - Length::from(50), Length::from(50));
        assert_eq!(Length::from(100) + Length::from(50), Length::from(150));

        assert_eq!(Length::from(100) + 50i32, Length::from(150));
        assert_eq!(Length::from(100) + 50u32, Length::from(150));
        assert_eq!(Length::from(100) + 50i64, Length::from(150));
        assert_eq!(Length::from(100) + 50u64, Length::from(150));
        assert_eq!(Length::from(100) + 50usize, Length::from(150));

        assert_eq!(Length::from(100) - 50i32, Length::from(50));
        assert_eq!(Length::from(100) - 50u32, Length::from(50));
        assert_eq!(Length::from(100) - 50i64, Length::from(50));
        assert_eq!(Length::from(100) - 50u64, Length::from(50));
        assert_eq!(Length::from(100) - 50usize, Length::from(50));
    }
}
