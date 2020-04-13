pub mod user;
pub use user::*;

pub mod kernel;
pub use kernel::*;

use crate::error::Result;

use flow_core::address::Address;
use flow_core::mem::AccessVirtualMemory;

pub trait Win32Process {
    fn wow64(&self) -> Address;
    fn peb(&self) -> Address;
    fn peb_module(&self) -> Address;

    fn peb_list<T: AccessVirtualMemory>(&self, mem: &mut T) -> Result<Vec<Address>>;
}
