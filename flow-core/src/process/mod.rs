pub mod emulator;

use crate::address::Address;
use crate::error::Result;

pub trait ProcessTrait {
    fn pid(&mut self) -> Result<i32>;
    fn name(&mut self) -> Result<String>;
    fn dtb(&mut self) -> Result<Address>;
}
