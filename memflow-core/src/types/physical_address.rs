/*!
Abstraction over a physical address with optional page information.
*/

use super::{Address, Page, PageType};

use std::fmt;

/// This type represents a wrapper over a [address](address/index.html)
/// with additional information about the containing page in the physical memory domain.
///
/// This type will mostly be used by the [virtual to physical address translation](todo.html).
/// When a physical address is translated from a virtual address the additional information
/// about the allocated page the virtual address points to can be obtained from this structure.
///
/// Most architectures have support multiple page sizes (see [huge pages](todo.html))
/// which will be represented by the containing `page` of the `PhysicalAddress` struct.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[repr(C)]
pub struct PhysicalAddress {
    address: Address,
    page_type: PageType,
    page_size_log2: u8,
}

/// Converts a `Address` into a `PhysicalAddress` with no page information attached.
impl From<Address> for PhysicalAddress {
    fn from(address: Address) -> Self {
        Self {
            address,
            page_type: PageType::UNKNOWN,
            page_size_log2: 0,
        }
    }
}

/// Constructs an `PhysicalAddress` from a `i32` value.
impl From<i32> for PhysicalAddress {
    fn from(item: i32) -> Self {
        Self::from(Address::from(item))
    }
}

/// Constructs an `PhysicalAddress` from a `u32` value.
impl From<u32> for PhysicalAddress {
    fn from(item: u32) -> Self {
        Self::from(Address::from(item))
    }
}

/// Constructs an `PhysicalAddress` from a `u64` value.
impl From<u64> for PhysicalAddress {
    fn from(item: u64) -> Self {
        Self::from(Address::from(item))
    }
}

/// Constructs an `PhysicalAddress` from a `usize` value.
impl From<usize> for PhysicalAddress {
    fn from(item: usize) -> Self {
        Self::from(Address::from(item))
    }
}

/// Converts a `PhysicalAddress` into a `Address`.
impl From<PhysicalAddress> for Address {
    fn from(address: PhysicalAddress) -> Self {
        Self::from(address.address.as_u64())
    }
}

impl PhysicalAddress {
    /// A physical address with a value of zero.
    pub const NULL: PhysicalAddress = PhysicalAddress {
        address: Address::null(),
        page_type: PageType::UNKNOWN,
        page_size_log2: 0,
    };

    /// A physical address with an invalid value.
    pub const INVALID: PhysicalAddress = PhysicalAddress {
        address: Address::INVALID,
        page_type: PageType::UNKNOWN,
        page_size_log2: 0,
    };

    /// Returns a physical address with a value of zero.
    #[inline]
    pub const fn null() -> Self {
        PhysicalAddress::NULL
    }

    /// Constructs a new `PhysicalAddress` form an `Address` with
    /// additional information about the page this address
    /// is contained in.
    ///
    /// Note: The page size must be a power of 2.
    #[inline]
    pub fn with_page(address: Address, page_type: PageType, page_size: usize) -> Self {
        Self {
            address,
            page_type,
            // TODO: this should be replaced by rust's internal functions as this is not endian aware
            // once it is stabilizied in rust
            // see issue: https://github.com/rust-lang/rust/issues/70887
            page_size_log2: (std::mem::size_of::<u64>() * 8
                - (page_size as u64).to_le().leading_zeros() as usize)
                as u8
                - 2,
        }
    }

    /// Checks wether the physical address is zero or not.
    #[inline]
    pub const fn is_null(&self) -> bool {
        self.address.is_null()
    }

    /// Returns a physical address that is invalid.
    #[inline]
    pub const fn invalid() -> Self {
        PhysicalAddress::INVALID
    }

    /// Checks wether the physical is valid or not.
    #[inline]
    pub const fn is_valid(&self) -> bool {
        self.address.is_valid()
    }

    /// Checks wether the physical address also contains page informations or not.
    #[inline]
    pub const fn has_page(&self) -> bool {
        self.page_size_log2 != 0
    }

    /// Returns the address of this physical address.
    #[inline]
    pub const fn address(&self) -> Address {
        self.address
    }

    /// Returns the type of page this physical address is contained in.
    #[inline]
    pub const fn page_type(&self) -> PageType {
        self.page_type
    }

    /// Returns the size of the page this physical address is contained in.
    #[inline]
    pub fn page_size(&self) -> usize {
        (2 << self.page_size_log2) as usize
    }

    /// Returns the base address of the containing page.
    pub fn page_base(&self) -> Address {
        if !self.has_page() {
            Address::INVALID
        } else {
            self.address.as_page_aligned(self.page_size())
        }
    }

    /// Converts the physical address into it's containing page page
    #[inline]
    pub fn containing_page(&self) -> Page {
        Page {
            page_type: self.page_type,
            page_base: self.page_base(),
            page_size: self.page_size(),
        }
    }

    /// Returns the containing address converted to a u32.
    #[inline]
    pub const fn as_u32(&self) -> u32 {
        self.address.as_u32()
    }

    /// Returns the internal u64 value of the address.
    #[inline]
    pub const fn as_u64(&self) -> u64 {
        self.address.as_u64()
    }

    /// Returns the containing address converted to a usize.
    #[inline]
    pub const fn as_usize(&self) -> usize {
        self.address.as_usize()
    }
}

/// Returns a physical address with a value of zero.
impl Default for PhysicalAddress {
    fn default() -> Self {
        Self::NULL
    }
}

impl fmt::Debug for PhysicalAddress {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:x}", self.address)
    }
}
impl fmt::UpperHex for PhysicalAddress {
    #[inline(always)]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:X}", self.address)
    }
}
impl fmt::LowerHex for PhysicalAddress {
    #[inline(always)]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:x}", self.address)
    }
}
impl fmt::Display for PhysicalAddress {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:x}", self.address)
    }
}

#[cfg(test)]
mod tests {
    use super::super::size;
    use super::*;

    #[test]
    fn test_page_size() {
        let pa = PhysicalAddress::with_page(Address::from(0x1234), PageType::UNKNOWN, 0x1000);
        assert_eq!(pa.page_size(), 0x1000);
        assert_eq!(pa.page_base(), Address::from(0x1000));
    }

    #[test]
    fn test_page_size_invalid() {
        let pa_42 = PhysicalAddress::with_page(Address::from(0x1234), PageType::UNKNOWN, 42);
        assert_ne!(pa_42.page_size(), 42);

        let pa_0 = PhysicalAddress::with_page(Address::from(0x1234), PageType::UNKNOWN, 42);
        assert_ne!(pa_0.page_size(), 0);
    }

    #[test]
    #[allow(clippy::unreadable_literal)]
    fn test_page_size_huge() {
        let pa_2mb =
            PhysicalAddress::with_page(Address::from(0x123456), PageType::UNKNOWN, size::mb(2));
        assert_eq!(pa_2mb.page_size(), size::mb(2));
        assert_eq!(pa_2mb.page_base(), Address::from(0));

        let pa_1gb =
            PhysicalAddress::with_page(Address::from(0x1234567), PageType::UNKNOWN, size::gb(1));
        assert_eq!(pa_1gb.page_size(), size::gb(1));
        assert_eq!(pa_1gb.page_base(), Address::from(0));
    }
}
