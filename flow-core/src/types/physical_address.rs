/*!
Abstraction over a physical address with optional page information.
*/

use super::{Address, Length, Page, PageType};

/**
This type represents a wrapper over a [address](address/index.html)
with additional information about the containing page in the physical memory domain.

This type will mostly be used by the [virtual to physical address translation](todo.html).
When a physical address is translated from a virtual address the additional information
about the allocated page the virtual address points to can be obtained from this structure.

Most architectures have support multiple page sizes (see [huge pages](todo.html))
which will be represented by the containing `page` of the `PhysicalAddress` struct.
*/
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct PhysicalAddress {
    pub address: Address,
    pub page_type: PageType,
    pub page_size: Length,
}

/// Converts a `Address` into a `PhysicalAddress` with no page information attached.
impl From<Address> for PhysicalAddress {
    fn from(address: Address) -> Self {
        Self {
            address,
            page_type: PageType::UNKNOWN,
            page_size: Length::ZERO,
        }
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
        page_size: Length::ZERO,
    };

    /// A physical address with an invalid value.
    pub const INVALID: PhysicalAddress = PhysicalAddress {
        address: Address::INVALID,
        page_type: PageType::UNKNOWN,
        page_size: Length::ZERO,
    };

    /// Returns a physical address with a value of zero.
    pub const fn null() -> Self {
        PhysicalAddress::NULL
    }

    /// Checks wether the physical address is zero or not.
    pub const fn is_null(&self) -> bool {
        self.address.is_null()
    }

    /// Returns a physical address that is invalid.
    pub const fn invalid() -> Self {
        PhysicalAddress::INVALID
    }

    /// Checks wether the physical is valid or not.
    pub const fn is_valid(&self) -> bool {
        self.address.is_valid()
    }

    /// Checks wether the physical address also contains page informations or not.
    pub const fn has_page(&self) -> bool {
        !self.page_size.is_zero()
    }

    /// Returns the base address of the containing page.
    pub fn page_base(&self) -> Address {
        if !self.has_page() {
            Address::INVALID
        } else {
            self.address.as_page_aligned(self.page_size)
        }
    }

    /// Converts the physical address into it's containing page page
    pub fn containing_page(&self) -> Page {
        Page {
            page_type: self.page_type,
            page_base: self.page_base(),
            page_size: self.page_size,
        }
    }

    /// Converts the physical address into an address.
    pub fn as_addr(&self) -> Address {
        self.address
    }

    /// Returns the containing address converted to a u32.
    pub const fn as_u32(&self) -> u32 {
        self.address.as_u32()
    }

    /// Returns the internal u64 value of the address.
    pub const fn as_u64(&self) -> u64 {
        self.address.as_u64()
    }

    /// Returns the containing address converted to a usize.
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
