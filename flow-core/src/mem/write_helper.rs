use crate::error::Result;

use crate::address::{Address, Length};

// TODO: add more helper funcs

pub trait VirtualWriteHelper {
    fn virt_read(&mut self, addr: Address, len: Length) -> Result<Vec<u8>>;
}
