//pub mod user;
//pub use user::UserProcess;

//pub mod kernel;
//pub use kernel::KernelProcess;

//pub mod user_iter;
//pub use user_iter::ProcessIterator;

// new temp module (will replace all the above)
pub mod win_user;
pub use win_user::*;

use crate::error::Result;

use crate::offsets::Win32Offsets;

use flow_core::address::Address;
use flow_core::mem::VirtualRead;

pub trait Win32Process {
    fn wow64(&self) -> Address;
    fn peb(&self) -> Address;
    fn peb_module(&self) -> Address;

    fn peb_list<T: VirtualRead>(&self, mem: &mut T, offsets: &Win32Offsets)
        -> Result<Vec<Address>>;
}
