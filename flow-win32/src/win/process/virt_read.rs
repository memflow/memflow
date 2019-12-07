use crate::error::{Error, Result};

use flow_core::address::{Address, Length};
use flow_core::mem::VirtualRead;
use flow_core::arch::{InstructionSet, Architecture};

use super::Process;

pub trait ProcessRead {
    fn virt_read_addr(&mut self, addr: Address) -> Result<Address>;
}

impl<T: VirtualRead> ProcessRead for Process<T> {
    fn virt_read_addr(&mut self, addr: Address) -> Result<Address> {
        let proc_arch = self.get_process_arch()?;
        let dtb = self.get_dtb()?;
        let win = self.win.borrow();
        let mem = &mut win.mem.borrow_mut();
        match proc_arch.instruction_set {
            InstructionSet::X64 => {
                Ok(mem.virt_read_addr64(
                    win.start_block.arch,
                    dtb,
                    addr)?)
            },
            InstructionSet::X86 => {
                Ok(mem.virt_read_addr32(
                    win.start_block.arch,
                    dtb,
                    addr)?)
            },
            _ => {
                Err(Error::new("invalid process architecture"))
            }
        }
    }
}