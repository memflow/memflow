use super::{Address, Length};

bitflags! {
    pub struct PageType: u8 {
        const UNKNOWN = 0b0000_0001;
        const PAGE_TABLE = 0b0000_0010;
        const WRITEABLE = 0b0000_0100;
        const READ_ONLY = 0b0000_1000;
    }
}

impl PageType {
    pub fn from_writeable_bit(writeable: bool) -> Self {
        if writeable {
            PageType::WRITEABLE
        } else {
            PageType::READ_ONLY
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct Page {
    pub page_type: PageType,
    pub page_base: Address,
    pub page_size: Length,
}
