//#[cfg(feature = "emulator")]
//pub mod emulator;

use crate::architecture::Architecture;
use crate::mem::*;
use crate::types::{Address, Length};

pub trait OperatingSystem {}

pub trait OsProcess {
    fn address(&self) -> Address;

    fn pid(&self) -> i32;
    fn name(&self) -> String;
    fn dtb(&self) -> Address;

    fn sys_arch(&self) -> Architecture;
    fn proc_arch(&self) -> Architecture;

    // virt_mem() - creates a VirtualMemory wrapper with system and process architecture
    fn virt_mem<'a, T: AccessVirtualMemory>(&self, mem: &'a mut T) -> VirtualMemoryContext<'a, T> {
        VirtualMemoryContext::with_proc_arch(mem, self.sys_arch(), self.proc_arch(), self.dtb())
    }
}

// TODO: Range impl for base to size?
pub trait OsProcessModule {
    fn address(&self) -> Address;
    fn parent_process(&self) -> Address;

    fn base(&self) -> Address;
    fn size(&self) -> Length;
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
