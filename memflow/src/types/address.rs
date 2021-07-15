/*!
Abstraction over a address on the target system.
*/

use crate::types::ByteSwap;
use core::convert::TryInto;
use std::default::Default;
use std::fmt;
use std::hash;
use std::ops;

/// The largest target memory type
#[allow(non_camel_case_types)]
pub type umem = u64;
#[allow(non_camel_case_types)]
pub type imem = i64;

/// `PrimitiveAddress` describes the address of a target system.
/// The current implementations include `u32`, `u64` and later eventually `u128`.
/// This trait can be used to abstract objects over the target pointer width.
pub trait PrimitiveAddress:
    Copy + Eq + PartialEq + Ord + PartialOrd + hash::Hash + fmt::LowerHex + fmt::UpperHex + ByteSwap
// + From<umem> + From<imem>
{
    fn null() -> Self;
    fn invalid() -> Self;

    fn min() -> Self;
    fn max() -> Self;

    fn wrapping_add(self, rhs: Self) -> Self;
    fn wrapping_sub(self, rhs: Self) -> Self;
    fn saturating_sub(self, rhs: Self) -> Self;
    fn overflowing_shr(self, rhs: u32) -> (Self, bool);

    fn to_umem(self) -> umem;
    fn to_imem(self) -> imem;

    #[inline]
    fn is_null(self) -> bool {
        self.eq(&Self::null())
    }
}

impl PrimitiveAddress for u32 {
    #[inline]
    fn null() -> Self {
        0u32
    }

    #[inline]
    fn invalid() -> Self {
        !0u32
    }

    #[inline]
    fn min() -> Self {
        u32::MIN
    }

    #[inline]
    fn max() -> Self {
        u32::MAX
    }

    #[inline]
    fn wrapping_add(self, rhs: Self) -> Self {
        self.wrapping_add(rhs)
    }

    #[inline]
    fn wrapping_sub(self, rhs: Self) -> Self {
        self.wrapping_sub(rhs)
    }

    #[inline]
    fn saturating_sub(self, rhs: Self) -> Self {
        self.saturating_sub(rhs)
    }

    #[inline]
    fn overflowing_shr(self, rhs: u32) -> (Self, bool) {
        self.overflowing_shr(rhs)
    }

    #[inline]
    fn to_umem(self) -> umem {
        self as umem
    }

    #[inline]
    fn to_imem(self) -> imem {
        self as imem
    }
}

impl PrimitiveAddress for u64 {
    #[inline]
    fn null() -> Self {
        0u64
    }

    #[inline]
    fn invalid() -> Self {
        !0u64
    }

    #[inline]
    fn min() -> Self {
        u64::MIN
    }

    #[inline]
    fn max() -> Self {
        u64::MAX
    }

    #[inline]
    fn wrapping_add(self, rhs: Self) -> Self {
        self.wrapping_add(rhs)
    }

    #[inline]
    fn wrapping_sub(self, rhs: Self) -> Self {
        self.wrapping_sub(rhs)
    }

    #[inline]
    fn saturating_sub(self, rhs: Self) -> Self {
        self.saturating_sub(rhs)
    }

    #[inline]
    fn overflowing_shr(self, rhs: u32) -> (Self, bool) {
        self.overflowing_shr(rhs)
    }

    #[inline]
    fn to_umem(self) -> umem {
        self as umem
    }

    #[inline]
    fn to_imem(self) -> imem {
        self as imem
    }
}

/// This type represents a address on the target system.
/// It internally holds a `umem` value but can also be used
/// when working in 32-bit environments.
///
/// This type will not handle overflow for 32-bit or 64-bit addresses / lengths.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[repr(transparent)]
pub struct Address(umem);

impl Address {
    /// A address with the value of zero.
    ///
    /// # Examples
    ///
    /// ```
    /// use memflow::types::Address;
    ///
    /// println!("address: {}", Address::NULL);
    /// ```
    pub const NULL: Address = Address { 0: 0 };

    /// A address with an invalid value.
    ///
    /// # Examples
    ///
    /// ```
    /// use memflow::types::Address;
    ///
    /// println!("address: {}", Address::INVALID);
    /// ```
    pub const INVALID: Address = Address { 0: !0 };

    /// Returns an address with a value of zero.
    ///
    /// # Examples
    ///
    /// ```
    /// use memflow::types::Address;
    ///
    /// println!("address: {}", Address::null());
    /// ```
    #[inline]
    pub const fn null() -> Self {
        Address::NULL
    }

    /// Creates a a bit mask.
    /// This function accepts an (half-open) range excluding the end bit from the mask.
    ///
    /// # Examples
    ///
    /// ```
    /// use memflow::types::Address;
    ///
    /// println!("mask: {}", Address::bit_mask(0..11));
    /// ```
    pub fn bit_mask<T: TryInto<umem>>(bits: ops::Range<T>) -> Address {
        ((0xffff_ffff_ffff_ffff >> (63 - bits.end.try_into().ok().unwrap()))
            & !(((1 as umem) << bits.start.try_into().ok().unwrap()) - 1))
            .into()
    }

    /// Creates a a bit mask (const version with u8 range).
    /// This function accepts an (half-open) range excluding the end bit from the mask.
    ///
    /// # Examples
    ///
    /// ```
    /// use memflow::types::Address;
    ///
    /// println!("mask: {}", Address::bit_mask(0..11));
    /// ```
    pub const fn bit_mask_u8(bits: ops::Range<u8>) -> Address {
        Address((0xffff_ffff_ffff_ffff >> (63 - bits.end)) & !(((1 as umem) << bits.start) - 1))
    }

    /// Checks wether the address is zero or not.
    ///
    /// # Examples
    ///
    /// ```
    /// use memflow::types::Address;
    ///
    /// assert_eq!(Address::null().is_null(), true);
    /// assert_eq!(Address::from(0x1000u64).is_null(), false);
    /// ```
    #[inline]
    pub const fn is_null(self) -> bool {
        self.0 == 0
    }

    /// Converts the address to an Option that is None when it is null
    ///
    /// # Examples
    ///
    /// ```
    /// use memflow::types::Address;
    ///
    /// assert_eq!(Address::null().non_null(), None);
    /// assert_eq!(Address::from(0x1000u64).non_null(), Some(Address::from(0x1000)));
    /// ```
    #[inline]
    pub fn non_null(self) -> Option<Address> {
        if self.is_null() {
            None
        } else {
            Some(self)
        }
    }

    /// Returns an address with a invalid value.
    ///
    /// # Examples
    ///
    /// ```
    /// use memflow::types::Address;
    ///
    /// println!("address: {}", Address::invalid());
    /// ```
    #[inline]
    pub const fn invalid() -> Self {
        Address::INVALID
    }

    /// Checks wether the address is valid or not.
    ///
    /// # Examples
    ///
    /// ```
    /// use memflow::types::Address;
    ///
    /// assert_eq!(Address::invalid().is_valid(), false);
    /// assert_eq!(Address::from(0x1000u64).is_valid(), true);
    /// ```
    #[inline]
    pub const fn is_valid(self) -> bool {
        self.0 != !0
    }

    /// Converts the address into a `u64` value.
    ///
    /// # Examples
    ///
    /// ```
    /// use memflow::types::{Address, umem};
    ///
    /// let addr = Address::from(0x1000u64);
    /// let addr_umem: umem = addr.to_umem();
    /// assert_eq!(addr_umem, 0x1000);
    /// ```
    #[inline]
    pub const fn to_umem(self) -> umem {
        self.0
    }

    /// Aligns the containing address to the given page size.
    /// It returns the base address of the containing page.
    ///
    /// # Examples
    ///
    /// ```
    /// use memflow::types::{Address, size};
    ///
    /// let addr = Address::from(0x1234);
    /// let aligned = addr.as_page_aligned(size::kb(4));
    /// assert_eq!(aligned, Address::from(0x1000));
    /// ```
    pub const fn as_page_aligned(self, page_size: umem) -> Address {
        Address {
            0: self.0 - self.0 % (page_size as umem),
        }
    }

    /// Returns true or false wether the bit at the specified index is either 0 or 1.
    /// An index of 0 will check the least significant bit.
    ///
    /// # Examples
    ///
    /// ```
    /// use memflow::types::Address;
    ///
    /// let addr = Address::from(2);
    /// let bit = addr.bit_at(1);
    /// assert_eq!(bit, true);
    /// ```
    pub const fn bit_at(self, idx: u8) -> bool {
        (self.0 & ((1 as umem) << idx)) != 0
    }

    /// Extracts the given range of bits by applying a corresponding bitmask.
    /// This function accepts an (half-open) range excluding the end bit from the mask.
    ///
    /// # Examples
    ///
    /// ```
    /// use memflow::types::Address;
    ///
    /// let addr = Address::from(123456789);
    /// println!("bits[0..2] = {}", addr.extract_bits(0..2));
    /// ```
    pub fn extract_bits<T: TryInto<umem>>(self, bits: ops::Range<T>) -> Address {
        (self.0 & Address::bit_mask(bits).to_umem()).into()
    }

    pub const fn wrapping_add(self, other: umem) -> Self {
        Self {
            0: self.0.wrapping_add(other as umem),
        }
    }

    pub const fn wrapping_sub(self, other: umem) -> Self {
        Self {
            0: self.0.wrapping_sub(other as umem),
        }
    }
}

/// Returns a address with a value of zero.
///
/// # Examples
///
/// ```
/// use memflow::types::Address;
///
/// assert_eq!(Address::default().is_null(), true);
/// ```
impl Default for Address {
    fn default() -> Self {
        Self::null()
    }
}

/// Implements byteswapping for the address
impl ByteSwap for Address {
    fn byte_swap(&mut self) {
        self.0.byte_swap();
    }
}

/// Constructs an `Address` from any value that implements [`PrimitiveAddress`].
impl<T: PrimitiveAddress> From<T> for Address {
    fn from(item: T) -> Self {
        Self { 0: item.to_umem() }
    }
}

/// Adds a `umem` to a `Address` which results in a `Address`.
/// # Examples
/// ```
/// use memflow::types::Address;
/// assert_eq!(Address::from(10) + 5usize, Address::from(15));
/// ```
impl ops::Add<umem> for Address {
    type Output = Self;

    fn add(self, other: umem) -> Self {
        Self {
            0: self.0 + (other as umem),
        }
    }
}

/// Adds any compatible type reference to Address
impl<'a, T: Into<umem> + Copy> ops::Add<&'a T> for Address {
    type Output = Self;

    fn add(self, other: &'a T) -> Self {
        Self {
            0: self.0 + (*other).into(),
        }
    }
}

/// Adds a `umem` to a `Address`.
///
/// # Examples
///
/// ```
/// use memflow::types::Address;
///
/// let mut addr = Address::from(10);
/// addr += 5;
/// assert_eq!(addr, Address::from(15));
/// ```
impl ops::AddAssign<umem> for Address {
    fn add_assign(&mut self, other: umem) {
        *self = Self {
            0: self.0 + (other as umem),
        }
    }
}

/// Subtracts a `Address` from a `Address` resulting in a `umem`.
///
/// # Examples
///
/// ```
/// use memflow::types::Address;
///
/// assert_eq!(Address::from(10) - 5, Address::from(5));
/// ```
impl ops::Sub for Address {
    type Output = umem;

    fn sub(self, other: Self) -> umem {
        (self.0 - other.0) as umem
    }
}

/// Subtracts a `umem` from a `Address` resulting in a `Address`.
impl ops::Sub<umem> for Address {
    type Output = Address;

    fn sub(self, other: umem) -> Address {
        Self {
            0: self.0 - (other as umem),
        }
    }
}

/// Subtracts any compatible type reference to Address
impl<'a, T: Into<umem> + Copy> ops::Sub<&'a T> for Address {
    type Output = Self;

    fn sub(self, other: &'a T) -> Self {
        Self {
            0: self.0 - (*other).into(),
        }
    }
}

/// Subtracts a `umem` from a `Address`.
///
/// # Examples
///
/// ```
/// use memflow::types::Address;
///
/// let mut addr = Address::from(10);
/// addr -= 5;
/// assert_eq!(addr, Address::from(5));
///
/// ```
impl ops::SubAssign<umem> for Address {
    fn sub_assign(&mut self, other: umem) {
        *self = Self {
            0: self.0 - (other as umem),
        }
    }
}

/// Adds a `usize` to a `Address` which results in a `Address`.
/// # Examples
/// ```
/// use memflow::types::Address;
/// assert_eq!(Address::from(10) + 5usize, Address::from(15));
/// ```
impl ops::Add<usize> for Address {
    type Output = Self;

    fn add(self, other: usize) -> Self {
        Self {
            0: self.0 + (other as umem),
        }
    }
}

/// Adds a `usize` to a `Address`.
///
/// # Examples
///
/// ```
/// use memflow::types::Address;
///
/// let mut addr = Address::from(10);
/// addr += 5;
/// assert_eq!(addr, Address::from(15));
/// ```
impl ops::AddAssign<usize> for Address {
    fn add_assign(&mut self, other: usize) {
        *self = Self {
            0: self.0 + (other as umem),
        }
    }
}

/// Subtracts a `usize` from a `Address` resulting in a `Address`.
impl ops::Sub<usize> for Address {
    type Output = Address;

    fn sub(self, other: usize) -> Address {
        Self {
            0: self.0 - (other as umem),
        }
    }
}

/// Subtracts a `usize` from a `Address`.
///
/// # Examples
///
/// ```
/// use memflow::types::Address;
///
/// let mut addr = Address::from(10);
/// addr -= 5;
/// assert_eq!(addr, Address::from(5));
///
/// ```
impl ops::SubAssign<usize> for Address {
    fn sub_assign(&mut self, other: usize) {
        *self = Self {
            0: self.0 - (other as umem),
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
    use super::super::size;
    use super::*;

    #[test]
    fn test_null_valid() {
        assert!(Address::null().is_null());
        assert!(!Address::invalid().is_valid());
    }

    #[test]
    fn test_from() {
        assert_eq!(Address::from(1337_u32).to_umem(), 1337);
        assert_eq!(Address::from(4321_u64).to_umem(), 4321);
    }

    #[test]
    fn test_alignment() {
        assert_eq!(
            Address::from(0x1234_u64).as_page_aligned(size::kb(4)),
            Address::from(0x1000_u64)
        );
        assert_eq!(
            Address::from(0xFFF1_2345_u64).as_page_aligned(0x10000),
            Address::from(0xFFF1_0000_u64)
        );
    }

    #[test]
    fn test_bits() {
        assert!(Address::from(1_u64).bit_at(0));
        assert!(!Address::from(1_u64).bit_at(1));
        assert!(!Address::from(1_u64).bit_at(2));
        assert!(!Address::from(1_u64).bit_at(3));

        assert!(!Address::from(2_u64).bit_at(0));
        assert!(Address::from(2_u64).bit_at(1));
        assert!(!Address::from(2_u64).bit_at(2));
        assert!(!Address::from(2_u64).bit_at(3));

        assert!(Address::from(13_u64).bit_at(0));
        assert!(!Address::from(13_u64).bit_at(1));
        assert!(Address::from(13_u64).bit_at(2));
        assert!(Address::from(13_u64).bit_at(3));
    }

    #[test]
    fn test_bit_mask() {
        assert_eq!(Address::bit_mask(0..11).to_umem(), 0xfff);
        assert_eq!(Address::bit_mask(12..20).to_umem(), 0x001f_f000);
        assert_eq!(Address::bit_mask(21..29).to_umem(), 0x3fe0_0000);
        assert_eq!(Address::bit_mask(30..38).to_umem(), 0x007f_c000_0000);
        assert_eq!(Address::bit_mask(39..47).to_umem(), 0xff80_0000_0000);
        assert_eq!(Address::bit_mask(12..51).to_umem(), 0x000f_ffff_ffff_f000);
    }

    #[test]
    fn test_ops() {
        assert_eq!(Address::from(10_u64) + 5usize, Address::from(15_u64));

        assert_eq!(Address::from(10_u64) - Address::from(5_u64), 5);
        assert_eq!(Address::from(100_u64) - 5usize, Address::from(95_u64));
    }
}
