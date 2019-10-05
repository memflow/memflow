use std::fmt;

// pub struct Length(u64)
#[derive(Debug, PartialEq, PartialOrd)]
pub struct Length {
    pub len: u64,
}

// TODO: impl traits for length
impl From<u64> for Length {
    fn from(item: u64) -> Self {
        Self{ len: item, }
    }
}

impl fmt::LowerHex for Length {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:x}", self.len)
    }
}
