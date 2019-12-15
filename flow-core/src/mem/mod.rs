pub mod read;
pub mod read_helper;
pub mod write;
pub mod write_helper;

pub use read::{PhysicalRead, VirtualRead};
pub use read_helper::{VirtualReadHelper, VirtualReadHelperFuncs, VirtualReader};
pub use write::{PhysicalWrite, VirtualWrite};

use crate::error::{Error, Result};
use std::mem;
use std::ptr::copy_nonoverlapping;

use byteorder::{BigEndian, ByteOrder, LittleEndian};

use crate::address::{Address, Length};
use crate::arch::{self, Architecture};

use std::ffi::CString;

macro_rules! arch_write_type {
    ($byte_order:expr, $func:ident, $buf:expr, $value:expr) => {
        match $byte_order {
            arch::ByteOrder::LittleEndian => LittleEndian::$func($buf, $value),
            arch::ByteOrder::BigEndian => BigEndian::$func($buf, $value),
        }
    };
}

/*
pub trait VirtualWrite {
    fn virt_write(
        &mut self,
        arch: Architecture,
        dtb: Address,
        addr: Address,
        data: &[u8],
    ) -> Result<Length>;

    fn virt_write_addr(
        &mut self,
        arch: Architecture,
        dtb: Address,
        addr: Address,
        val: Address,
    ) -> Result<Length> {
        let mut buf = vec![0; arch.instruction_set.len_addr().as_usize()];
        arch_write_type!(
            arch.instruction_set.byte_order(),
            write_u64,
            &mut buf,
            val.as_u64()
        );
        self.virt_write(arch, dtb, addr, &buf)
    }

    fn virt_write_u64(
        &mut self,
        arch: Architecture,
        dtb: Address,
        addr: Address,
        val: u64,
    ) -> Result<Length> {
        let mut buf = vec![0; arch.instruction_set.len_u64().as_usize()];
        arch_write_type!(arch.instruction_set.byte_order(), write_u64, &mut buf, val);
        self.virt_write(arch, dtb, addr, &buf)
    }

    fn virt_write_u32(
        &mut self,
        arch: Architecture,
        dtb: Address,
        addr: Address,
        val: u32,
    ) -> Result<Length> {
        let mut buf = vec![0; arch.instruction_set.len_u32().as_usize()];
        arch_write_type!(arch.instruction_set.byte_order(), write_u32, &mut buf, val);
        self.virt_write(arch, dtb, addr, &buf)
    }

    fn virt_write_i64(
        &mut self,
        arch: Architecture,
        dtb: Address,
        addr: Address,
        val: i64,
    ) -> Result<Length> {
        let mut buf = vec![0; arch.instruction_set.len_i64().as_usize()];
        arch_write_type!(arch.instruction_set.byte_order(), write_i64, &mut buf, val);
        self.virt_write(arch, dtb, addr, &buf)
    }

    fn virt_write_i32(
        &mut self,
        arch: Architecture,
        dtb: Address,
        addr: Address,
        val: i32,
    ) -> Result<Length> {
        let mut buf = vec![0; arch.instruction_set.len_i32().as_usize()];
        arch_write_type!(arch.instruction_set.byte_order(), write_i32, &mut buf, val);
        self.virt_write(arch, dtb, addr, &buf)
    }

    fn virt_write_f32(
        &mut self,
        arch: Architecture,
        dtb: Address,
        addr: Address,
        val: f32,
    ) -> Result<Length> {
        let mut buf = vec![0; arch.instruction_set.len_f32().as_usize()];
        arch_write_type!(arch.instruction_set.byte_order(), write_f32, &mut buf, val);
        self.virt_write(arch, dtb, addr, &buf)
    }
}
*/