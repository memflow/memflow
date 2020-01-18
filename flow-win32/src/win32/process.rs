pub mod user;
pub use user::*;

pub mod kernel;
pub use kernel::*;

use crate::error::Result;

use crate::offsets::Win32Offsets;

use flow_core::address::Address;
use flow_core::mem::VirtualMemoryTrait;

pub trait Win32Process {
    fn wow64(&self) -> Address;
    fn peb(&self) -> Address;
    fn peb_module(&self) -> Address;

    fn peb_list<T: VirtualMemoryTrait>(
        &self,
        mem: &mut T,
        offsets: &Win32Offsets,
    ) -> Result<Vec<Address>>;
}
