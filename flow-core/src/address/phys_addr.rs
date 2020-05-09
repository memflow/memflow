use super::{Address, Length, Page};

// PhysicalAddress - represents a physical address with additional paging info
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct PhysicalAddress {
    pub address: Address,
    pub page: Option<Page>,
}

impl From<Address> for PhysicalAddress {
    fn from(address: Address) -> Self {
        Self {
            address,
            page: None,
        }
    }
}

impl From<PhysicalAddress> for Address {
    fn from(address: PhysicalAddress) -> Self {
        Self::from(address.address.as_u64())
    }
}

// forward declares from addr
impl PhysicalAddress {
    pub const NULL: PhysicalAddress = PhysicalAddress {
        address: Address::null(),
        page: None,
    };

    pub const INVALID: PhysicalAddress = PhysicalAddress {
        address: Address::INVALID,
        page: None,
    };

    pub const fn null() -> Self {
        Self {
            address: Address::null(),
            page: None,
        }
    }

    pub const fn is_null(&self) -> bool {
        self.address.is_null()
    }

    pub fn as_addr(&self) -> Address {
        self.address
    }

    pub fn as_len(&self) -> Length {
        self.address.as_len()
    }

    pub const fn as_u32(&self) -> u32 {
        self.address.as_u32()
    }

    pub const fn as_u64(&self) -> u64 {
        self.address.as_u64()
    }

    pub const fn as_usize(&self) -> usize {
        self.address.as_usize()
    }
}

impl Default for PhysicalAddress {
    fn default() -> Self {
        Self::null()
    }
}
