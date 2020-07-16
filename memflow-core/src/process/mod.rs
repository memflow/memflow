use std::prelude::v1::*;

//#[cfg(feature = "emulator")]
//pub mod emulator;

use crate::architecture::Architecture;
use crate::types::Address;

pub trait OperatingSystem {}

pub trait OsProcessInfo {
    fn address(&self) -> Address;

    fn pid(&self) -> i32;
    fn name(&self) -> String;
    fn dtb(&self) -> Address;

    fn sys_arch(&self) -> Architecture;
    fn proc_arch(&self) -> Architecture;
}

// TODO: Range impl for base to size?
pub trait OsProcessModuleInfo {
    fn address(&self) -> Address;
    fn parent_process(&self) -> Address;

    fn base(&self) -> Address;
    fn size(&self) -> usize;
    fn name(&self) -> String;
}

// TODO: refactor? or something
/*
pub trait ExportTrait {
    fn name(&self) -> &str;
    fn offset(&self) -> Length;
}

pub trait SectionTrait {
    fn name(&self) -> &str;
    fn virt_addr(&self) -> Address;
    fn virt_size(&self) -> Length;
}
*/
