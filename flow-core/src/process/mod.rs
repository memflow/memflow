#[cfg(feature = "emulator")]
pub mod emulator;

use crate::address::{Address, Length};
use crate::arch::Architecture;
use crate::error::Result;
use crate::ida_pattern::*;
use crate::mem::*;

// TODO:
pub trait OperatingSystem {}

// TODO: add more?
pub trait ProcessTrait {
    fn address(&self) -> Address;

    fn pid(&self) -> i32;
    fn name(&self) -> String;
    fn dtb(&self) -> Address;

    fn sys_arch(&self) -> Architecture;
    fn proc_arch(&self) -> Architecture;
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
    fn address(&self) -> Address;
    fn parent_process(&self) -> Address;

    fn base(&self) -> Address;
    fn size(&self) -> Length;
    fn name(&self) -> String;
}

pub trait ExportTrait {
    fn name(&self) -> &str;
    fn offset(&self) -> Length;
}

pub trait SectionTrait {
    fn name(&self) -> &str;
    fn virt_addr(&self) -> Address;
    fn virt_size(&self) -> Length;
}

pub trait FindSignatureTrait {
    fn signature(&mut self, pattern: &str) -> Result<Length>;
}

impl<T> FindSignatureTrait for T
where
    T: ModuleTrait + VirtualReadHelper,
{
    fn signature(&mut self, pattern: &str) -> Result<Length> {
        let base = self.base();
        let size = self.size();

        let buf = self.virt_read(base, size)?;
        let m = pattern.try_match_ida_regex(&buf[..])?;

        Ok(len!(m.0))
    }
}
