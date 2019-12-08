use std::fmt;
use std::ops;

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Length(u64);

impl fmt::LowerHex for Length {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:x}", self.0)
    }
}

// TODO: sort them by likeliness
impl From<u16> for Length {
    fn from(item: u16) -> Self {
        Self { 0: u64::from(item) }
    }
}

impl From<i16> for Length {
    fn from(item: i16) -> Self {
        Self { 0: item as u64 }
    }
}

impl From<u32> for Length {
    fn from(item: u32) -> Self {
        Self { 0: u64::from(item) }
    }
}

impl From<i32> for Length {
    fn from(item: i32) -> Self {
        Self { 0: item as u64 }
    }
}

impl From<u64> for Length {
    fn from(item: u64) -> Self {
        Self { 0: item }
    }
}

impl From<i64> for Length {
    fn from(item: i64) -> Self {
        Self { 0: item as u64 }
    }
}

impl From<usize> for Length {
    fn from(item: usize) -> Self {
        Self { 0: item as u64 }
    }
}

impl Length {
    pub fn zero() -> Self {
        Length::from(0)
    }

    pub fn is_zero(self) -> bool {
        self.0 == 0
    }

    pub fn as_u64(self) -> u64 {
        self.0
    }

    pub fn as_usize(self) -> usize {
        self.0 as usize
    }

    pub fn from_b(len: u64) -> Self {
        Length { 0: len }
    }

    pub fn from_kb(len: u64) -> Self {
        Length { 0: len * 1024 }
    }

    pub fn from_kib(len: u64) -> Self {
        Length { 0: len * 1024 * 8 }
    }

    pub fn from_mb(len: u64) -> Self {
        Length {
            0: len * 1024 * 1024,
        }
    }

    pub fn from_mib(len: u64) -> Self {
        Length {
            0: len * 1024 * 1024 * 8,
        }
    }

    pub fn from_gb(len: u64) -> Self {
        Length {
            0: len * 1024 * 1024 * 1024,
        }
    }

    pub fn from_gib(len: u64) -> Self {
        Length {
            0: len * 1024 * 1024 * 1024 * 8,
        }
    }
}

// Length + Length => Length
impl ops::Add for Length {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            0: self.0 + other.0,
        }
    }
}

// Length + i32 => Length
impl ops::Add<i32> for Length {
    type Output = Self;

    fn add(self, other: i32) -> Self {
        Self {
            0: self.0 + (other as u64),
        }
    }
}

// Length + u32 => Length
impl ops::Add<u32> for Length {
    type Output = Self;

    fn add(self, other: u32) -> Self {
        Self {
            0: self.0 + (u64::from(other)),
        }
    }
}

// Length + i64 => Length
impl ops::Add<i64> for Length {
    type Output = Self;

    fn add(self, other: i64) -> Self {
        Self {
            0: self.0 + (other as u64),
        }
    }
}

// Length + u64 => Length
impl ops::Add<u64> for Length {
    type Output = Self;

    fn add(self, other: u64) -> Self {
        Self { 0: self.0 + other }
    }
}

// Length + usize => Length
impl ops::Add<usize> for Length {
    type Output = Self;

    fn add(self, other: usize) -> Self {
        Self {
            0: self.0 + (other as u64),
        }
    }
}

// Length * usize = Length
impl ops::Mul<usize> for Length {
    type Output = Self;

    fn mul(self, other: usize) -> Self {
        Self {
            0: self.0 * (other as u64),
        }
    }
}

// Length += Length
impl ops::AddAssign for Length {
    fn add_assign(&mut self, other: Self) {
        *self = Self {
            0: self.0 + other.0,
        }
    }
}

// Length += i32
impl ops::AddAssign<i32> for Length {
    fn add_assign(&mut self, other: i32) {
        *self = Self {
            0: self.0 + (other as u64),
        }
    }
}

// Length += u32
impl ops::AddAssign<u32> for Length {
    fn add_assign(&mut self, other: u32) {
        *self = Self {
            0: self.0 + (u64::from(other)),
        }
    }
}

// Length += i64
impl ops::AddAssign<i64> for Length {
    fn add_assign(&mut self, other: i64) {
        *self = Self {
            0: self.0 + (other as u64),
        }
    }
}

// Length += u64
impl ops::AddAssign<u64> for Length {
    fn add_assign(&mut self, other: u64) {
        *self = Self { 0: self.0 + other }
    }
}

// Length += usize
impl ops::AddAssign<usize> for Length {
    fn add_assign(&mut self, other: usize) {
        *self = Self {
            0: self.0 + (other as u64),
        }
    }
}

// Length - Length => Length
impl ops::Sub for Length {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self {
            0: self.0 - other.0,
        }
    }
}

// Length - i32 => Length
impl ops::Sub<i32> for Length {
    type Output = Self;

    fn sub(self, other: i32) -> Self {
        Self {
            0: self.0 - (other as u64),
        }
    }
}

// Length - u32 => Length
impl ops::Sub<u32> for Length {
    type Output = Self;

    fn sub(self, other: u32) -> Self {
        Self {
            0: self.0 - (u64::from(other)),
        }
    }
}

// Length - i64 => Length
impl ops::Sub<i64> for Length {
    type Output = Self;

    fn sub(self, other: i64) -> Self {
        Self {
            0: self.0 - (other as u64),
        }
    }
}

// Length - u64 => Length
impl ops::Sub<u64> for Length {
    type Output = Self;

    fn sub(self, other: u64) -> Self {
        Self { 0: self.0 - other }
    }
}

// Length - usize => Length
impl ops::Sub<usize> for Length {
    type Output = Self;

    fn sub(self, other: usize) -> Self {
        Self {
            0: self.0 - (other as u64),
        }
    }
}

// Length -= Length
impl ops::SubAssign for Length {
    fn sub_assign(&mut self, other: Self) {
        *self = Self {
            0: self.0 - other.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::address::Length;

    #[test]
    fn test_from() {
        assert_eq!(Length::zero().as_u64(), 0);
        assert_eq!(Length::from(1337).as_u64(), 1337);
        assert_eq!(Length::from(4321).as_usize(), 4321);
        assert_eq!(Length::from_b(500), Length::from(500));
        assert_eq!(Length::from_kb(20), Length::from(20 * 1024));
        assert_eq!(Length::from_kib(123), Length::from(123 * 1024 * 8));
        assert_eq!(Length::from_mb(20), Length::from(20 * 1024 * 1024));
        assert_eq!(Length::from_mib(52), Length::from(52 * 1024 * 1024 * 8));
        assert_eq!(
            Length::from_gb(20),
            Length::from(20u64 * 1024 * 1024 * 1024)
        );
        assert_eq!(
            Length::from_gib(52),
            Length::from(52u64 * 1024 * 1024 * 1024 * 8)
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
