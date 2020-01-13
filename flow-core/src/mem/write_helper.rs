use crate::error::Result;

use std::mem;

use byteorder::{BigEndian, ByteOrder, LittleEndian};

use crate::address::{Address, Length};
use crate::arch::{self, Architecture, ArchitectureTrait};
use crate::mem::*;

// TODO: add more helper funcs

pub trait VirtualWriteHelper {
    fn virt_write(&mut self, addr: Address, data: &[u8]) -> Result<Length>;
}

// VirtualWriter will wrap all helper methods
pub struct VirtualWriter<'a, T: VirtualWrite> {
    writer: &'a mut T,
    sys_arch: Architecture,
    type_arch: Architecture,
    dtb: Address,
}

impl<'a, T: VirtualWrite> VirtualWriter<'a, T> {
    pub fn with(writer: &'a mut T, sys_arch: Architecture, dtb: Address) -> Self {
        Self {
            writer,
            sys_arch,
            type_arch: sys_arch,
            dtb,
        }
    }

    pub fn with_type_arch(
        writer: &'a mut T,
        sys_arch: Architecture,
        type_arch: Architecture,
        dtb: Address,
    ) -> Self {
        Self {
            writer,
            sys_arch,
            type_arch,
            dtb,
        }
    }
}

impl<'a, T: VirtualWrite> ArchitectureTrait for VirtualWriter<'a, T> {
    fn arch(&mut self) -> Result<Architecture> {
        Ok(self.sys_arch)
    }
}

impl<'a, T: VirtualWrite> TypeArchitectureTrait for VirtualWriter<'a, T> {
    fn type_arch(&mut self) -> Result<Architecture> {
        Ok(self.type_arch)
    }
}

impl<'a, T: VirtualWrite> VirtualWriteHelper for VirtualWriter<'a, T> {
    fn virt_write(&mut self, addr: Address, data: &[u8]) -> Result<Length> {
        self.writer.virt_write(self.sys_arch, self.dtb, addr, data)
    }
}

// VirtualWriteHelperFuncs
pub trait VirtualWriteHelperFuncs {
    //unsafe fn virt_write_raw<U>(&mut self, addr: Address, data: U) -> Result<Length>;

    //fn virt_write_addr(&mut self, addr: Address, data: &[u8]) -> Result<Length>;

    //fn virt_write_vec_addr(&mut self, addr: Address, data: Vec<Address>) -> Result<Length>;

    fn virt_write_addr32(&mut self, addr: Address, data: Address) -> Result<Length>;

    fn virt_write_vec_addr32(&mut self, addr: Address, data: Vec<Address>) -> Result<Length>;

    fn virt_write_addr64(&mut self, addr: Address, data: Address) -> Result<Length>;

    fn virt_write_vec_addr64(&mut self, addr: Address, data: Vec<Address>) -> Result<Length>;

    fn virt_write_u64(&mut self, addr: Address, data: u64) -> Result<Length>;

    fn virt_write_u32(&mut self, addr: Address, data: u32) -> Result<Length>;

    fn virt_write_u16(&mut self, addr: Address, data: u16) -> Result<Length>;

    fn virt_write_u8(&mut self, addr: Address, data: u8) -> Result<Length>;

    fn virt_write_i64(&mut self, addr: Address, data: i64) -> Result<Length>;

    fn virt_write_i32(&mut self, addr: Address, data: i32) -> Result<Length>;

    fn virt_write_i16(&mut self, addr: Address, data: i16) -> Result<Length>;

    fn virt_write_i8(&mut self, addr: Address, data: i8) -> Result<Length>;

    fn virt_write_f32(&mut self, addr: Address, data: f32) -> Result<Length>;

    // TODO: add more vec read helpers
    fn virt_write_vec_f32(&mut self, addr: Address, data: Vec<f32>) -> Result<Length>;

    //fn virt_write_cstr(&mut self, addr: Address, data: &str) -> Result<Length>;
}

macro_rules! arch_write_type {
    ($addr:expr, $vec:expr, $byte_order:expr, $func:ident, $value:expr) => {
        match $byte_order {
            arch::ByteOrder::LittleEndian => LittleEndian::$func($vec, $value),
            arch::ByteOrder::BigEndian => BigEndian::$func($vec, $value),
        }
    };
}

macro_rules! arch_write_vec_type {
    ($byte_order:expr, $elem_size:expr, $func:ident, $value:expr) => {
        $value
            .into_iter()
            .flat_map(|v| {
                let mut u = vec![0_u8; $elem_size.as_usize()];
                match $byte_order {
                    arch::ByteOrder::LittleEndian => LittleEndian::$func(&mut u[..], v),
                    arch::ByteOrder::BigEndian => BigEndian::$func(&mut u[..], v),
                };
                u
            })
            .collect::<Vec<u8>>()
    };
}

impl<T> VirtualWriteHelperFuncs for T
where
    T: VirtualWriteHelper + ArchitectureTrait + TypeArchitectureTrait,
{
    /*unsafe fn virt_write_raw<U>(&mut self, addr: Address, data: U) -> Result<Length> {
        let s = Length::size_of::<U>();
        let v = Vec::from(&data as *mut _ as *mut u8, s, s);
        self.virt_write(addr, &v)
    }*/

    // TODO: arch/type_arch agnostic
    //fn virt_write_addr(&mut self, addr: Address, data: &[u8]) -> Result<Length>;

    //fn virt_write_vec_addr(&mut self, addr: Address, data: Vec<Address>) -> Result<Length>;

    fn virt_write_addr32(&mut self, addr: Address, data: Address) -> Result<Length> {
        self.virt_write_u32(addr, data.as_u32())
    }

    fn virt_write_vec_addr32(&mut self, addr: Address, data: Vec<Address>) -> Result<Length> {
        let ta = self.type_arch()?;
        let v = arch_write_vec_type!(
            ta.instruction_set.byte_order(),
            Length::size_of::<u32>(),
            write_u32,
            data.into_iter().map(Address::as_u32).collect::<Vec<u32>>()
        );
        self.virt_write(addr, &v[..])
    }

    fn virt_write_addr64(&mut self, addr: Address, data: Address) -> Result<Length> {
        self.virt_write_u64(addr, data.as_u64())
    }

    fn virt_write_vec_addr64(&mut self, addr: Address, data: Vec<Address>) -> Result<Length> {
        let ta = self.type_arch()?;
        let v = arch_write_vec_type!(
            ta.instruction_set.byte_order(),
            Length::size_of::<u64>(),
            write_u64,
            data.into_iter().map(Address::as_u64).collect::<Vec<u64>>()
        );
        self.virt_write(addr, &v[..])
    }

    fn virt_write_u64(&mut self, addr: Address, data: u64) -> Result<Length> {
        let ta = self.type_arch()?;
        let mut v = vec![0_u8; mem::size_of::<u64>()];
        arch_write_type!(
            addr,
            &mut v[..],
            ta.instruction_set.byte_order(),
            write_u64,
            data
        );
        self.virt_write(addr, &v)
    }

    fn virt_write_u32(&mut self, addr: Address, data: u32) -> Result<Length> {
        let ta = self.type_arch()?;
        let mut v = vec![0_u8; mem::size_of::<u32>()];
        arch_write_type!(
            addr,
            &mut v[..],
            ta.instruction_set.byte_order(),
            write_u32,
            data
        );
        self.virt_write(addr, &v)
    }

    fn virt_write_u16(&mut self, addr: Address, data: u16) -> Result<Length> {
        let ta = self.type_arch()?;
        let mut v = vec![0_u8; mem::size_of::<u16>()];
        arch_write_type!(
            addr,
            &mut v[..],
            ta.instruction_set.byte_order(),
            write_u16,
            data
        );
        self.virt_write(addr, &v)
    }

    fn virt_write_u8(&mut self, addr: Address, data: u8) -> Result<Length> {
        let v = vec![data, 1];
        self.virt_write(addr, &v)
    }

    fn virt_write_i64(&mut self, addr: Address, data: i64) -> Result<Length> {
        let ta = self.type_arch()?;
        let mut v = vec![0_u8; mem::size_of::<i64>()];
        arch_write_type!(
            addr,
            &mut v[..],
            ta.instruction_set.byte_order(),
            write_i64,
            data
        );
        self.virt_write(addr, &v)
    }

    fn virt_write_i32(&mut self, addr: Address, data: i32) -> Result<Length> {
        let ta = self.type_arch()?;
        let mut v = vec![0_u8; mem::size_of::<i32>()];
        arch_write_type!(
            addr,
            &mut v[..],
            ta.instruction_set.byte_order(),
            write_i32,
            data
        );
        self.virt_write(addr, &v)
    }

    fn virt_write_i16(&mut self, addr: Address, data: i16) -> Result<Length> {
        let ta = self.type_arch()?;
        let mut v = vec![0_u8; mem::size_of::<i16>()];
        arch_write_type!(
            addr,
            &mut v[..],
            ta.instruction_set.byte_order(),
            write_i16,
            data
        );
        self.virt_write(addr, &v)
    }

    fn virt_write_i8(&mut self, addr: Address, data: i8) -> Result<Length> {
        let v = vec![data as u8, 1];
        self.virt_write(addr, &v)
    }

    fn virt_write_f32(&mut self, addr: Address, data: f32) -> Result<Length> {
        let ta = self.type_arch()?;
        let mut v = vec![0_u8; mem::size_of::<f32>()];
        arch_write_type!(
            addr,
            &mut v[..],
            ta.instruction_set.byte_order(),
            write_f32,
            data
        );
        self.virt_write(addr, &v)
    }

    // TODO: add more vec read helpers
    fn virt_write_vec_f32(&mut self, addr: Address, data: Vec<f32>) -> Result<Length> {
        let ta = self.type_arch()?;
        let v = arch_write_vec_type!(
            ta.instruction_set.byte_order(),
            Length::size_of::<f32>(),
            write_f32,
            data
        );
        self.virt_write(addr, &v[..])
    }

    //fn virt_write_cstr(&mut self, addr: Address, data: &str) -> Result<Length>;
}
