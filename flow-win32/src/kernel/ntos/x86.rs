use crate::error::{Error, Result};

use flow_core::address::{Address, Length};
use flow_core::mem::AccessVirtualMemory;

pub fn find<T: AccessVirtualMemory>(_mem: &mut T) -> Result<(Address, Length)> {
    Err(Error::new("find_x86(): not implemented yet"))
}
