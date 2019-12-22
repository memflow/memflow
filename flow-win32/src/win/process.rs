pub mod user;
pub use user::UserProcess;

pub mod kernel;
pub use kernel::KernelProcess;

pub mod user_iter;
pub use user_iter::ProcessIterator;

use crate::error::Result;

use super::Windows;

use flow_core::address::Address;
use flow_core::arch::SystemArchitecture;
use flow_core::mem::*;

use crate::win::module::{Module, ModuleIterator};
use crate::win::unicode_string::VirtualReadUnicodeString;

pub trait ProcessTrait {
    fn pid(&mut self) -> Result<i32>;
    fn name(&mut self) -> Result<String>;
    fn dtb(&mut self) -> Result<Address>;

    fn first_peb_entry(&mut self) -> Result<Address>;
    fn module_iter(&self) -> Result<ModuleIterator<Self>>
    where
        Self: Sized + SystemArchitecture + VirtualReadHelperFuncs + VirtualReadUnicodeString;
}

pub trait ProcessModuleHelper {
    fn module(&self, name: &str) -> Result<Module<Self>>
    where
        Self: Sized
            + ProcessTrait
            + SystemArchitecture
            + VirtualReadHelperFuncs
            + VirtualReadUnicodeString;
}

impl<T> ProcessModuleHelper for T
where
    T: Sized
        + ProcessTrait
        + SystemArchitecture
        + VirtualReadHelperFuncs
        + VirtualReadUnicodeString,
{
    fn module(&self, name: &str) -> Result<Module<Self>> {
        Ok(self
            .module_iter()?
            .filter_map(|mut m| {
                if m.name().unwrap_or_default() == name {
                    Some(m)
                } else {
                    None
                }
            })
            .nth(0)
            .ok_or_else(|| "unable to find module")?)
    }
}
