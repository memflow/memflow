/*!
Abstraction over a address on the target system.
*/

use super::{PhysicalAddress, Pointer};
use crate::types::ByteSwap;

use core::convert::TryInto;
use std::default::Default;
use std::fmt;
use std::hash;
use std::ops;

/// The largest target memory type
/// The following core rule is defined for these memory types:
///
/// `PAGE_SIZE < usize <= umem`
///
/// Where `PAGE_SIZE` is any lowest granularity page size, `usize` is the standard size type, and
/// `umem` is memflow's memory size type.
///
/// This means that `usize` can always be safely cast to `umem`, while anything to do with page
/// sizes can be cast to `umem` safely,
///
#[cfg(feature = "64_bit_mem")]
#[allow(non_camel_case_types)]
pub type umem = u64;
#[cfg(all(feature = "128_bit_mem", not(feature = "64_bit_mem")))]
#[allow(non_camel_case_types)]
pub type umem = u128;
#[cfg(all(not(feature = "64_bit_mem"), not(feature = "128_bit_mem")))]
#[allow(non_camel_case_types)]
pub type umem = usize;
#[cfg(feature = "64_bit_mem")]
#[allow(non_camel_case_types)]
pub type imem = i64;
#[cfg(all(feature = "128_bit_mem", not(feature = "64_bit_mem")))]
#[allow(non_camel_case_types)]
pub type imem = i128;
#[cfg(all(not(feature = "64_bit_mem"), not(feature = "128_bit_mem")))]
#[allow(non_camel_case_types)]
pub type imem = isize;

pub const UMEM_BITS: u8 = core::mem::size_of::<umem>() as u8 * 8;

// Enforce the `umem` >= `usize` condition. Whenever a real 128-bit architecture is here, `umem`
// should be expanded to 128 bits.
const _: [u8; (core::mem::size_of::<usize>() <= core::mem::size_of::<umem>()) as usize] = [0; 1];

pub const fn clamp_to_usize(val: umem) -> usize {
    let max = core::usize::MAX as umem;

    let ret = if max < val { max } else { val };

    ret as usize
}

pub const fn clamp_to_isize(val: imem) -> isize {
    let max = core::isize::MAX as imem;
    let min = core::isize::MIN as imem;

    let ret = if max < val {
        max
    } else if min > val {
        min
    } else {
        val
    };

    ret as isize
}

/// `PrimitiveAddress` describes the address of a target system.
/// The current implementations include `u32`, `u64` and later eventually `u128`.
/// This trait can be used to abstract objects over the target pointer width.
pub trait PrimitiveAddress:
    Copy
    + Eq
    + PartialEq
    + Ord
    + PartialOrd
    + hash::Hash
    + fmt::LowerHex
    + fmt::UpperHex
    + ByteSwap
    + ops::Add<Output = Self>
    + ops::Sub<Output = Self>
{
    fn null() -> Self;
    fn invalid() -> Self;

    fn min() -> Self;
    fn max() -> Self;

    fn from_umem(frm: umem) -> Self;
    fn from_imem(frm: imem) -> Self;

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

#[macro_export]
macro_rules! impl_primitive_address {
    ($type_name:ident) => {
        impl PrimitiveAddress for $type_name {
            #[inline]
            fn null() -> Self {
                0 as $type_name
            }

            #[inline]
            fn invalid() -> Self {
                !Self::null()
            }

            #[inline]
            fn min() -> Self {
                Self::MIN
            }

            #[inline]
            fn max() -> Self {
                Self::MAX
            }

            #[inline]
            fn from_umem(frm: umem) -> Self {
                frm as Self
            }

            #[inline]
            fn from_imem(frm: imem) -> Self {
                frm as Self
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
    };
}

impl_primitive_address!(u16);
impl_primitive_address!(u32);
impl_primitive_address!(u64);
#[cfg(all(feature = "128_bit_mem", not(feature = "64_bit_mem")))]
impl_primitive_address!(u128);

/// This type represents a address on the target system.
/// It internally holds a `umem` value but can also be used
/// when working in 32-bit environments.
///
/// This type will not handle overflow for 32-bit or 64-bit addresses / lengths.
#[repr(transparent)]
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[cfg_attr(feature = "abi_stable", derive(::abi_stable::StableAbi))]
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
    pub const NULL: Address = Address(0);

    /// A address with an invalid value.
    ///
    /// # Examples
    ///
    /// ```
    /// use memflow::types::Address;
    ///
    /// println!("address: {}", Address::INVALID);
    /// ```
    pub const INVALID: Address = Address(!0);

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
    /// println!("mask: {}", Address::bit_mask(0..=11));
    /// ```
    pub fn bit_mask<T: TryInto<u8>>(bits: ops::RangeInclusive<T>) -> Address
    where
        T: TryInto<u8>,
        T: Copy,
    {
        Address(
            (!0 >> ((UMEM_BITS - 1) - (*bits.end()).try_into().ok().unwrap()))
                & !((1 << (*bits.start()).try_into().ok().unwrap()) - 1),
        )
    }

    /// Creates a a bit mask (const version with u8 range).
    /// This function accepts an (half-open) range excluding the end bit from the mask.
    ///
    /// # Examples
    ///
    /// ```
    /// use memflow::types::Address;
    ///
    /// println!("mask: {}", Address::bit_mask_u8(0..=11));
    /// ```
    pub const fn bit_mask_u8(bits: ops::RangeInclusive<u8>) -> Address {
        Address((!0 >> (UMEM_BITS - 1 - *bits.end())) & !((1 << *bits.start()) - 1))
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
    /// use memflow::types::{Address, mem};
    ///
    /// let addr = Address::from(0x1234);
    /// let aligned = addr.as_mem_aligned(mem::kb(4));
    /// assert_eq!(aligned, Address::from(0x1000));
    /// ```
    pub const fn as_mem_aligned(self, mem_size: umem) -> Self {
        Self(self.0 - self.0 % mem_size)
    }

    pub const fn as_page_aligned(self, page_size: usize) -> Self {
        self.as_mem_aligned(page_size as umem)
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
        (self.0 & (1 << idx)) != 0
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
    /// println!("bits[0..2] = {}", addr.extract_bits(0..=2));
    /// ```
    pub fn extract_bits<T: TryInto<u8>>(self, bits: ops::RangeInclusive<T>) -> Address
    where
        T: Copy,
    {
        (self.0 & Address::bit_mask(bits).to_umem()).into()
    }

    /// Wrapping (modular) addition. Computes `self + rhs`,
    /// wrapping around at the boundary of the type.
    pub const fn wrapping_add(self, other: Self) -> Self {
        Self(self.0.wrapping_add(other.0))
    }

    /// Wrapping (modular) subtraction. Computes `self - rhs`,
    /// wrapping around at the boundary of the type.
    pub const fn wrapping_sub(self, other: Self) -> Self {
        Self(self.0.wrapping_sub(other.0))
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

#[macro_export]
macro_rules! impl_address_from {
    ($type_name:ident) => {
        impl From<$type_name> for Address {
            fn from(item: $type_name) -> Self {
                Self { 0: item as umem }
            }
        }

        impl<T: ?Sized> From<Pointer<$type_name, T>> for Address {
            #[inline(always)]
            fn from(ptr: Pointer<$type_name, T>) -> Self {
                Self {
                    0: ptr.inner as umem,
                }
            }
        }
    };
}

// u16, u32, u64 is handled by the PrimitiveAddress implementation below.
impl_address_from!(i8);
impl_address_from!(u8);
impl_address_from!(i16);
//impl_address_from!(u16);
impl_address_from!(i32);
//impl_address_from!(u32);
impl_address_from!(i64);
//impl_address_from!(u64);
impl_address_from!(usize);
#[cfg(all(feature = "128_bit_mem", not(feature = "64_bit_mem")))]
impl_address_from!(i128);

/// Converts any `PrimitiveAddress` into an Address.
impl<U: PrimitiveAddress> From<U> for Address {
    #[inline(always)]
    fn from(val: U) -> Self {
        Self(val.to_umem())
    }
}

/// Converts a `PhysicalAddress` into a `Address`.
impl From<PhysicalAddress> for Address {
    fn from(address: PhysicalAddress) -> Self {
        address.address
    }
}

/// Converts any `Pointer` into an Address.
impl<U: PrimitiveAddress, T: ?Sized> From<Pointer<U, T>> for Address {
    #[inline(always)]
    fn from(ptr: Pointer<U, T>) -> Self {
        Self(ptr.inner.to_umem())
    }
}

#[macro_export]
macro_rules! impl_address_arithmetic_unsigned {
    ($type_name:ident) => {
        impl ops::Add<$type_name> for Address {
            type Output = Self;

            fn add(self, other: $type_name) -> Self {
                Self {
                    0: self.0 + (other as umem),
                }
            }
        }

        impl ops::AddAssign<$type_name> for Address {
            fn add_assign(&mut self, other: $type_name) {
                *self = Self {
                    0: self.0 + (other as umem),
                }
            }
        }

        impl ops::Sub<$type_name> for Address {
            type Output = Address;

            fn sub(self, other: $type_name) -> Address {
                Self {
                    0: self.0 - (other as umem),
                }
            }
        }

        impl ops::SubAssign<$type_name> for Address {
            fn sub_assign(&mut self, other: $type_name) {
                *self = Self {
                    0: self.0 - (other as umem),
                }
            }
        }
    };
}

#[macro_export]
macro_rules! impl_address_arithmetic_signed {
    ($type_name:ident) => {
        impl ops::Add<$type_name> for Address {
            type Output = Self;

            fn add(self, other: $type_name) -> Self {
                if other >= 0 {
                    Self {
                        0: self.0 + (other as umem),
                    }
                } else {
                    Self {
                        0: self.0 - (-other as umem),
                    }
                }
            }
        }

        impl ops::AddAssign<$type_name> for Address {
            fn add_assign(&mut self, other: $type_name) {
                if other >= 0 {
                    *self = Self {
                        0: self.0 + (other as umem),
                    }
                } else {
                    *self = Self {
                        0: self.0 - (-other as umem),
                    }
                }
            }
        }

        impl ops::Sub<$type_name> for Address {
            type Output = Address;

            fn sub(self, other: $type_name) -> Address {
                if other >= 0 {
                    Self {
                        0: self.0 - (other as umem),
                    }
                } else {
                    Self {
                        0: self.0 + (-other as umem),
                    }
                }
            }
        }

        impl ops::SubAssign<$type_name> for Address {
            fn sub_assign(&mut self, other: $type_name) {
                if other >= 0 {
                    *self = Self {
                        0: self.0 - (other as umem),
                    }
                } else {
                    *self = Self {
                        0: self.0 + (-other as umem),
                    }
                }
            }
        }
    };
}

impl_address_arithmetic_signed!(i8);
impl_address_arithmetic_signed!(i16);
impl_address_arithmetic_signed!(i32);
impl_address_arithmetic_signed!(i64);
#[cfg(all(feature = "128_bit_mem", not(feature = "64_bit_mem")))]
impl_address_arithmetic_signed!(i128);
impl_address_arithmetic_signed!(isize);
impl_address_arithmetic_unsigned!(u8);
impl_address_arithmetic_unsigned!(u16);
impl_address_arithmetic_unsigned!(u32);
impl_address_arithmetic_unsigned!(u64);
#[cfg(all(feature = "128_bit_mem", not(feature = "64_bit_mem")))]
impl_address_arithmetic_unsigned!(u128);
impl_address_arithmetic_unsigned!(usize);

/// Adds any compatible type reference to Address
impl<'a, T: Into<umem> + Copy> ops::Add<&'a T> for Address {
    type Output = Self;

    fn add(self, other: &'a T) -> Self {
        Self(self.0 + (*other).into())
    }
}

/// Subtracts any compatible type reference to Address
impl<'a, T: Into<umem> + Copy> ops::Sub<&'a T> for Address {
    type Output = Self;

    fn sub(self, other: &'a T) -> Self {
        Self(self.0 - (*other).into())
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
    type Output = imem;

    fn sub(self, other: Self) -> imem {
        if self.0 > other.0 {
            (self.0 - other.0) as imem
        } else {
            -((other.0 - self.0) as imem)
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
        assert_eq!(Address::bit_mask(0..=11).to_umem(), 0xfff);
        assert_eq!(Address::bit_mask(12..=20).to_umem(), 0x001f_f000);
        assert_eq!(Address::bit_mask(21..=29).to_umem(), 0x3fe0_0000);
        assert_eq!(Address::bit_mask(30..=38).to_umem(), 0x007f_c000_0000);
        assert_eq!(Address::bit_mask(39..=47).to_umem(), 0xff80_0000_0000);
        assert_eq!(Address::bit_mask(12..=51).to_umem(), 0x000f_ffff_ffff_f000);
    }

    #[test]
    fn test_bit_mask_u8() {
        assert_eq!(Address::bit_mask_u8(0..=11).to_umem(), 0xfff);
        assert_eq!(Address::bit_mask_u8(12..=20).to_umem(), 0x001f_f000);
        assert_eq!(Address::bit_mask_u8(21..=29).to_umem(), 0x3fe0_0000);
        assert_eq!(Address::bit_mask_u8(30..=38).to_umem(), 0x007f_c000_0000);
        assert_eq!(Address::bit_mask_u8(39..=47).to_umem(), 0xff80_0000_0000);
        assert_eq!(
            Address::bit_mask_u8(12..=51).to_umem(),
            0x000f_ffff_ffff_f000
        );
    }

    #[test]
    fn test_ops() {
        assert_eq!(Address::from(10_u64) + 5usize, Address::from(15_u64));

        assert_eq!(Address::from(10_u64) - Address::from(5_u64), 5);
        assert_eq!(Address::from(100_u64) - 5usize, Address::from(95_u64));
    }
}
