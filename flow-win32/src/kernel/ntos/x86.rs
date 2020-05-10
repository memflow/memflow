use crate::error::{Error, Result};

use flow_core::mem::AccessVirtualMemory;
use flow_core::types::{Address, Length};

pub fn find<T: AccessVirtualMemory>(_mem: &mut T) -> Result<(Address, Length)> {
    Err(Error::new("find_x86(): not implemented yet"))
}
