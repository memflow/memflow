/*!
This module contains data structures related to information about a page.
*/

use super::Address;

bitflags! {
    /// Describes the type of a page using a bitflag.
    #[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
    #[repr(transparent)]
    pub struct PageType: u8 {
        /// The page explicitly has no flags.
        const NONE = 0b0000_0000;
        /// The page type is not known.
        const UNKNOWN = 0b0000_0001;
        /// The page contains page table entries.
        const PAGE_TABLE = 0b0000_0010;
        /// The page is a writeable page.
        const WRITEABLE = 0b0000_0100;
        /// The page is read only.
        const READ_ONLY = 0b0000_1000;
        /// The page is not executable.
        const NOEXEC = 0b0001_0000;
    }
}

impl PageType {
    pub fn write(mut self, flag: bool) -> Self {
        self &= !(PageType::WRITEABLE | PageType::READ_ONLY | PageType::UNKNOWN);
        if flag {
            self | PageType::WRITEABLE
        } else {
            self | PageType::READ_ONLY
        }
    }

    pub fn noexec(mut self, flag: bool) -> Self {
        self &= !(PageType::NOEXEC);
        if flag {
            self | PageType::NOEXEC
        } else {
            self
        }
    }

    pub fn page_table(mut self, flag: bool) -> Self {
        self &= !(PageType::PAGE_TABLE | PageType::UNKNOWN);
        if flag {
            self | PageType::PAGE_TABLE
        } else {
            self
        }
    }
}

impl Default for PageType {
    fn default() -> Self {
        PageType::UNKNOWN
    }
}

/// A `Page` holds information about a memory page.
///
/// More information about paging can be found [here](https://en.wikipedia.org/wiki/Paging).
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct Page {
    /// Contains the page type (see above).
    pub page_type: PageType,
    /// Contains the base address of this page.
    pub page_base: Address,
    /// Contains the size of this page.
    pub page_size: usize,
}

impl Page {
    /// A page object that is invalid.
    pub const INVALID: Page = Page {
        page_type: PageType::UNKNOWN,
        page_base: Address::INVALID,
        page_size: 0,
    };

    /// Returns a page that is invalid.
    pub const fn invalid() -> Self {
        Self::INVALID
    }

    /// Checks wether the page is valid or not.
    pub fn is_valid(&self) -> bool {
        self.page_base.is_valid() && self.page_size != 0
    }
}
