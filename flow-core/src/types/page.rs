/*!
This module contains data structures related to information about a page.
*/

use super::{Address, Length};

bitflags! {
    /// Describes the type of a page using a bitflag.
    pub struct PageType: u8 {
        /// The page type is not known.
        const UNKNOWN = 0b0000_0001;
        /// The page contains page table entries.
        const PAGE_TABLE = 0b0000_0010;
        /// The page is a writeable page.
        const WRITEABLE = 0b0000_0100;
        /// The page is read only.
        const READ_ONLY = 0b0000_1000;
    }
}

// TODO: removeme - this is not very ergonomic
impl PageType {
    pub fn from_writeable_bit(writeable: bool) -> Self {
        if writeable {
            PageType::WRITEABLE
        } else {
            PageType::READ_ONLY
        }
    }
}

/**
A `Page` holds information about a memory page.

More information about paging can be found [here](https://en.wikipedia.org/wiki/Paging).
*/
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct Page {
    /// Contains the page type (see above).
    pub page_type: PageType,
    /// Contains the base address of this page.
    pub page_base: Address,
    /// Contains the size of this page.
    pub page_size: Length,
}

impl Page {
    /// A page object that is invalid.
    pub const INVALID: Page = Page {
        page_type: PageType::UNKNOWN,
        page_base: Address::INVALID,
        page_size: Length::zero(),
    };

    /// Returns a page that is invalid.
    pub const fn invalid() -> Self {
        Self::INVALID
    }
}
