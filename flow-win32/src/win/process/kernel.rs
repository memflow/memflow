use crate::error::Result;

use super::Windows;

use flow_core::address::{Address, Length};
use flow_core::arch::{Architecture, ArchitectureTrait};
use flow_core::mem::*;
use flow_core::process::ProcessTrait;

use std::cell::RefCell;
use std::rc::Rc;

use crate::win::module::ModuleIterator;
use crate::win::process::ProcessModuleTrait;

pub struct KernelProcess<T: VirtualRead> {
    pub win: Rc<RefCell<Windows<T>>>,
    pub module_list: Address,
}

impl<T: VirtualRead> Clone for KernelProcess<T>
where
    Rc<RefCell<T>>: Clone,
    Address: Clone,
{
    fn clone(&self) -> Self {
        Self {
            win: self.win.clone(),
            module_list: self.module_list,
        }
    }
}

// TODO: everything that implements module iter should get some help funcs (find_module, etc)
impl<T: VirtualRead> KernelProcess<T> {
    pub fn with(win: Rc<RefCell<Windows<T>>>, module_list: Address) -> Self {
        Self { win, module_list }
    }
}

impl<T: VirtualRead> ProcessTrait for KernelProcess<T> {
    fn pid(&mut self) -> flow_core::Result<i32> {
        Ok(0)
    }

    // system arch = type arch
    fn name(&mut self) -> flow_core::Result<String> {
        Ok("ntoskrnl.exe".to_string())
    }

    // system arch = type arch
    fn dtb(&mut self) -> flow_core::Result<Address> {
        let win = self.win.borrow();
        Ok(win.start_block.dtb)
    }
}

impl<T: VirtualRead> ProcessModuleTrait for KernelProcess<T> {
    fn first_peb_entry(&mut self) -> Result<Address> {
        Ok(self.module_list)
    }

    // module_iter will explicitly clone self and feed it into an iterator
    fn module_iter(&self) -> Result<ModuleIterator<KernelProcess<T>>> {
        let rc = Rc::new(RefCell::new(self.clone()));
        ModuleIterator::new(rc)
    }
}

// rename ArchitectureTrait -> ArchitectureTrait
impl<T: VirtualRead> ArchitectureTrait for KernelProcess<T> {
    fn arch(&mut self) -> flow_core::Result<Architecture> {
        let win = self.win.borrow();
        Ok(win.start_block.arch)
    }
}

// rename TypeArchitectureTrait -> TypeArchitectureTrait
impl<T: VirtualRead> TypeArchitectureTrait for KernelProcess<T> {
    fn type_arch(&mut self) -> flow_core::Result<Architecture> {
        self.arch()
    }
}

// TODO: this is not entirely correct as it will use different VAT than required, split vat arch + type arch up again
impl<T: VirtualRead> VirtualReadHelper for KernelProcess<T> {
    fn virt_read(&mut self, addr: Address, len: Length) -> flow_core::Result<Vec<u8>> {
        let proc_arch = self.arch().map_err(flow_core::Error::new)?;
        let dtb = self.dtb().map_err(flow_core::Error::new)?;
        let win = self.win.borrow();
        let mem = &mut win.mem.borrow_mut();
        mem.virt_read(proc_arch, dtb, addr, len)
    }
}
