use crate::error::{Error, Result};

use flow_core::address::{Address, Length};
use flow_core::mem::VirtualRead;
use flow_core::arch::{InstructionSet, Architecture};

use super::Process;

// TODO: rename mem_call_read
macro_rules! mem_call_read {
    ($sel:ident, $func:ident, $addr:expr) => {
        {
            let proc_arch = $sel.get_process_arch()?;
            let dtb = $sel.get_dtb()?;
            let win = $sel.win.borrow();
            let mem = &mut win.mem.borrow_mut();
            Ok(mem.$func(
                proc_arch,
                dtb,
                $addr)?)
        }
    };
}

macro_rules! mem_call_vec_read {
    ($sel:ident, $func:ident, $addr:expr, $count:expr) => {
        {
            let proc_arch = $sel.get_process_arch()?;
            let dtb = $sel.get_dtb()?;
            let win = $sel.win.borrow();
            let mem = &mut win.mem.borrow_mut();
            Ok(mem.$func(
                proc_arch,
                dtb,
                $addr,
                $count)?)
        }
    };
}

// addr, addr32, addr64, u64, u32, u16, u8, i64, i32, i16, i8, f32, cstr
pub trait ProcessRead {
    fn virt_read_addr(&mut self, addr: Address) -> Result<Address>;
    fn virt_read_vec_addr(&mut self, addr: Address, count: usize) -> Result<Vec<Address>>;

    fn virt_read_u64(&mut self, addr: Address) -> Result<u64>;
    fn virt_read_u32(&mut self, addr: Address) -> Result<u32>;
    fn virt_read_u16(&mut self, addr: Address) -> Result<u16>;
    fn virt_read_u8(&mut self, addr: Address) -> Result<u8>;
    fn virt_read_i64(&mut self, addr: Address) -> Result<i64>;
    fn virt_read_i32(&mut self, addr: Address) -> Result<i32>;
    fn virt_read_i16(&mut self, addr: Address) -> Result<i16>;
    fn virt_read_i8(&mut self, addr: Address) -> Result<i8>;

    fn virt_read_f32(&mut self, addr: Address) -> Result<f32>;
    fn virt_read_vec_f32(&mut self, addr: Address, count: usize) -> Result<Vec<f32>>;

    fn virt_read_cstr(&mut self, addr: Address, count: usize) -> Result<String>;
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

    fn virt_read_vec_addr(&mut self, addr: Address, count: usize) -> Result<Vec<Address>> {
        let proc_arch = self.get_process_arch()?;
        let dtb = self.get_dtb()?;
        let win = self.win.borrow();
        let mem = &mut win.mem.borrow_mut();
        match proc_arch.instruction_set {
            InstructionSet::X64 => {
                Ok(mem.virt_read_vec_addr64(
                    win.start_block.arch,
                    dtb,
                    addr,
                    count)?)
            },
            InstructionSet::X86 => {
                Ok(mem.virt_read_vec_addr32(
                    win.start_block.arch,
                    dtb,
                    addr,
                    count)?)
            },
            _ => {
                Err(Error::new("invalid process architecture"))
            }
        }
    }

    fn virt_read_u64(&mut self, addr: Address) -> Result<u64> {
        mem_call_read!(self, virt_read_u64, addr)
    }

    fn virt_read_u32(&mut self, addr: Address) -> Result<u32> {
        mem_call_read!(self, virt_read_u32, addr)
    }

    fn virt_read_u16(&mut self, addr: Address) -> Result<u16> {
        mem_call_read!(self, virt_read_u16, addr)
    }

    fn virt_read_u8(&mut self, addr: Address) -> Result<u8> {
        mem_call_read!(self, virt_read_u8, addr)
    }

    fn virt_read_i64(&mut self, addr: Address) -> Result<i64> {
        mem_call_read!(self, virt_read_i64, addr)
    }

    fn virt_read_i32(&mut self, addr: Address) -> Result<i32> {
        mem_call_read!(self, virt_read_i32, addr)
    }

    fn virt_read_i16(&mut self, addr: Address) -> Result<i16> {
        mem_call_read!(self, virt_read_i16, addr)
    }

    fn virt_read_i8(&mut self, addr: Address) -> Result<i8> {
        mem_call_read!(self, virt_read_i8, addr)
    }

    fn virt_read_f32(&mut self, addr: Address) -> Result<f32> {
        mem_call_read!(self, virt_read_f32, addr)
    }

    fn virt_read_vec_f32(&mut self, addr: Address, count: usize) -> Result<Vec<f32>> {
        mem_call_vec_read!(self, virt_read_vec_f32, addr, count)
    }

    fn virt_read_cstr(&mut self, addr: Address, len: usize) -> Result<String> {
        let proc_arch = self.get_process_arch()?;
        let dtb = self.get_dtb()?;
        let win = self.win.borrow();
        let mem = &mut win.mem.borrow_mut();
        Ok(mem.virt_read_cstr(
            proc_arch,
            dtb,
            addr,
            len)?) // TODO: get rid of this Ok() wrap? (move error class to core!)
    }
}

pub trait ProcessReadChain {
    // TODO: weak chain?
    fn virt_read_addr_chain(&mut self, base_addr: Address, offsets: Vec<Length>) -> Result<Address>;
}

// TODO: more error checking?
impl<T: ProcessRead> ProcessReadChain for T {
    fn virt_read_addr_chain(&mut self, base_addr: Address, offsets: Vec<Length>) -> Result<Address> {
        offsets.iter().try_fold(base_addr, |c, &a| self.virt_read_addr(c + a))
    }
}
