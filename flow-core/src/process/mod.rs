#[cfg(feature = "emulator")]
pub mod emulator;

use crate::address::{Address, Length};
use crate::error::Result;

// TODO: add more?
pub trait ProcessTrait {
    fn pid(&mut self) -> Result<i32>;
    fn name(&mut self) -> Result<String>;
    fn dtb(&mut self) -> Result<Address>;
}

// TODO: ProcessIterTrait

// TODO: generic Iterator
/*
pub trait ModuleIterTrait {
    fn module_iter(&self) -> Result<ModuleIterator<Self>>
    where
        Self: Sized + ArchitectureTrait + VirtualReadHelper + VirtualReadHelperFuncs;
}
*/

// TODO: Range impl for base to size?
// TODO: add more?
// TODO: maybe remove mut and fetch when module is loaded?
pub trait ModuleTrait {
    fn base(&mut self) -> Result<Address>;
    fn size(&mut self) -> Result<Length>;
    fn name(&mut self) -> Result<String>;
}

pub trait ExportTrait {
    fn name(&self) -> String;
    fn offset(&self) -> Length;
}