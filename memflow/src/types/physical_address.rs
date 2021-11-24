/*!
Abstraction over a physical address with optional page information.
*/

use super::{umem, Address, Page, PageType, Pointer, PrimitiveAddress};

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
#[repr(C)]
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[cfg_attr(feature = "abi_stable", derive(::abi_stable::StableAbi))]
pub struct PhysicalAddress {
    pub address: Address,
    pub page_type: PageType,
    page_size_log2: u8,
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
    pub fn with_page(address: Address, page_type: PageType, page_size: umem) -> Self {
        Self {
            address,
            page_type,
            // TODO: this should be replaced by rust's internal functions as this is not endian aware
            // once it is stabilizied in rust
            // see issue: https://github.com/rust-lang/rust/issues/70887
            page_size_log2: (std::mem::size_of::<umem>() * 8
                - page_size.to_le().leading_zeros() as usize) as u8
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
    pub fn page_size(&self) -> umem {
        (2 << self.page_size_log2) as umem
    }

    /// Returns the base address of the containing page.
    pub fn page_base(&self) -> Address {
        if !self.has_page() {
            Address::INVALID
        } else {
            self.address.as_mem_aligned(self.page_size())
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

    /// Returns the containing address converted to a raw [`umem`].
    #[inline]
    pub const fn to_umem(self) -> umem {
        self.address.to_umem()
    }
}

/// Returns a physical address with a value of zero.
impl Default for PhysicalAddress {
    fn default() -> Self {
        Self::NULL
    }
}

#[macro_export]
macro_rules! impl_physical_address_from {
    ($type_name:ident) => {
        impl From<$type_name> for PhysicalAddress {
            fn from(item: $type_name) -> Self {
                Self {
                    address: (item as umem).into(),
                    page_type: PageType::UNKNOWN,
                    page_size_log2: 0,
                }
            }
        }

        impl<T: ?Sized> From<Pointer<$type_name, T>> for PhysicalAddress {
            #[inline(always)]
            fn from(ptr: Pointer<$type_name, T>) -> Self {
                Self {
                    address: (ptr.inner as umem).into(),
                    page_type: PageType::UNKNOWN,
                    page_size_log2: 0,
                }
            }
        }
    };
}

// u16, u32, u64 is handled by the PrimitiveAddress implementation below.
impl_physical_address_from!(i8);
impl_physical_address_from!(u8);
impl_physical_address_from!(i16);
//impl_physical_address_from!(u16);
impl_physical_address_from!(i32);
//impl_physical_address_from!(u32);
impl_physical_address_from!(i64);
//impl_physical_address_from!(u64);
impl_physical_address_from!(usize);

impl<U: PrimitiveAddress> From<U> for PhysicalAddress {
    #[inline(always)]
    fn from(val: U) -> Self {
        Self {
            address: val.to_umem().into(),
            page_type: PageType::UNKNOWN,
            page_size_log2: 0,
        }
    }
}

impl<U: PrimitiveAddress, T: ?Sized> From<Pointer<U, T>> for PhysicalAddress {
    #[inline(always)]
    fn from(ptr: Pointer<U, T>) -> Self {
        Self {
            address: ptr.inner.to_umem().into(),
            page_type: PageType::UNKNOWN,
            page_size_log2: 0,
        }
    }
}

/// Converts a `PhysicalAddress` into a `Address`.
impl From<Address> for PhysicalAddress {
    fn from(address: Address) -> Self {
        Self {
            address,
            page_type: PageType::UNKNOWN,
            page_size_log2: 0,
        }
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
    use super::super::mem;
    use super::*;

    #[test]
    fn test_page_size() {
        let pa = PhysicalAddress::with_page(Address::from(0x1234_u64), PageType::UNKNOWN, 0x1000);
        assert_eq!(pa.page_size(), 0x1000);
        assert_eq!(pa.page_base(), Address::from(0x1000_u64));
    }

    #[test]
    fn test_page_size_invalid() {
        let pa_42 = PhysicalAddress::with_page(Address::from(0x1234_u64), PageType::UNKNOWN, 42);
        assert_ne!(pa_42.page_size(), 42);

        let pa_0 = PhysicalAddress::with_page(Address::from(0x1234_u64), PageType::UNKNOWN, 42);
        assert_ne!(pa_0.page_size(), 0);
    }

    #[test]
    #[allow(clippy::unreadable_literal)]
    fn test_page_size_huge() {
        let pa_2mb =
            PhysicalAddress::with_page(Address::from(0x123456_u64), PageType::UNKNOWN, mem::mb(2));
        assert_eq!(pa_2mb.page_size(), mem::mb(2));
        assert_eq!(pa_2mb.page_base(), Address::from(0_u64));

        let pa_1gb =
            PhysicalAddress::with_page(Address::from(0x1234567_u64), PageType::UNKNOWN, mem::gb(1));
        assert_eq!(pa_1gb.page_size(), mem::gb(1));
        assert_eq!(pa_1gb.page_base(), Address::from(0_u64));
    }
}
