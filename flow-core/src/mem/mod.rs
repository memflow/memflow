use crate::address::{Address, Length};
use crate::arch::{self, Architecture};
use crate::Result;

use byteorder::{BigEndian, ByteOrder, LittleEndian};
use std::ffi::CString;
use std::mem;

// generic traits
pub trait PhysicalMemoryTrait {
    fn phys_read(&mut self, addr: Address, out: &mut [u8]) -> Result<()>;
    fn phys_write(&mut self, addr: Address, data: &[u8]) -> Result<()>;

    fn phys_read_ret(&mut self, addr: Address, len: Length) -> Result<Vec<u8>> {
        let mut buf = vec![0; len.as_usize()];
        self.phys_read(addr, &mut buf)?;
        Ok(buf)
    }
}

pub trait VirtualMemoryTrait {
    fn virt_read(
        &mut self,
        arch: Architecture,
        dtb: Address,
        addr: Address,
        out: &mut [u8],
    ) -> Result<()>;

    fn virt_write(
        &mut self,
        arch: Architecture,
        dtb: Address,
        addr: Address,
        data: &[u8],
    ) -> Result<()>;

    fn virt_read_ret(
        &mut self,
        arch: Architecture,
        dtb: Address,
        addr: Address,
        len: Length,
    ) -> Result<Vec<u8>> {
        let mut buf = vec![0; len.as_usize()];
        self.virt_read(arch, dtb, addr, &mut buf)?;
        Ok(buf)
    }
}

pub struct VirtualMemory<'a, T: VirtualMemoryTrait> {
    mem: &'a mut T,
    sys_arch: Architecture,
    type_arch: Architecture,
    dtb: Address,
}

macro_rules! arch_read_type {
    ($byte_order:expr, $func:ident, $value:expr) => {
        match $byte_order {
            arch::ByteOrder::LittleEndian => LittleEndian::$func($value),
            arch::ByteOrder::BigEndian => BigEndian::$func($value),
        }
    };
}

macro_rules! arch_read_vec_type {
    ($byte_order:expr, $elem_size:expr, $type:ident, $func:ident, $value:expr) => {
        match $byte_order {
            arch::ByteOrder::LittleEndian => $value
                .chunks($elem_size.as_usize())
                .into_iter()
                .map(|v| LittleEndian::$func(v))
                .collect::<Vec<$type>>(),
            arch::ByteOrder::BigEndian => $value
                .chunks($elem_size.as_usize())
                .into_iter()
                .map(|v| BigEndian::$func(v))
                .collect::<Vec<$type>>(),
        }
    };
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

impl<'a, T: VirtualMemoryTrait> VirtualMemory<'a, T> {
    pub fn with(mem: &'a mut T, sys_arch: Architecture, dtb: Address) -> Self {
        Self {
            mem,
            sys_arch,
            type_arch: sys_arch,
            dtb,
        }
    }

    pub fn with_type_arch(
        mem: &'a mut T,
        sys_arch: Architecture,
        type_arch: Architecture,
        dtb: Address,
    ) -> Self {
        Self {
            mem,
            sys_arch,
            type_arch,
            dtb,
        }
    }

    pub fn sys_arch(&self) -> Architecture {
        self.sys_arch
    }

    pub fn type_arch(&self) -> Architecture {
        self.type_arch
    }

    pub fn dtb(&self) -> Address {
        self.dtb
    }

    pub fn virt_read(&mut self, addr: Address, out: &mut [u8]) -> Result<()> {
        self.mem.virt_read(self.sys_arch, self.dtb, addr, out)
    }

    pub fn virt_write(&mut self, addr: Address, data: &[u8]) -> Result<()> {
        self.mem.virt_write(self.sys_arch, self.dtb, addr, data)
    }

    pub fn virt_read_ret(&mut self, addr: Address, len: Length) -> Result<Vec<u8>> {
        self.mem.virt_read_ret(self.sys_arch, self.dtb, addr, len)
    }

    // TODO: replace these with nice pod trait! :)
    // read helpers
    pub fn virt_read_addr(&mut self, addr: Address) -> Result<Address> {
        let r = self.virt_read_ret(addr, self.type_arch.instruction_set.len_addr())?;
        Ok(Address::from(arch_read_type!(
            self.type_arch.instruction_set.byte_order(),
            read_u64, // TODO: make this architecture agnostic! (might crash here)
            &r
        )))
    }

    pub fn virt_read_vec_addr(&mut self, addr: Address, count: usize) -> Result<Vec<Address>> {
        let r = self.virt_read_ret(addr, self.type_arch.instruction_set.len_addr() * count)?;
        Ok(arch_read_vec_type!(
            self.type_arch.instruction_set.byte_order(),
            self.type_arch.instruction_set.len_addr(),
            u64,
            read_u64, // TODO: make this architecture agnostic! (might crash here)
            &r
        )
        .into_iter()
        .map(Address::from)
        .collect())
    }

    pub fn virt_read_addr32(&mut self, addr: Address) -> Result<Address> {
        Ok(addr!(self.virt_read_u32(addr)?))
    }

    pub fn virt_read_vec_addr32(&mut self, addr: Address, count: usize) -> Result<Vec<Address>> {
        let r = self.virt_read_ret(addr, Length::size_of::<u32>() * count)?;
        Ok(arch_read_vec_type!(
            self.type_arch.instruction_set.byte_order(),
            Length::size_of::<u32>(),
            u32,
            read_u32,
            &r
        )
        .into_iter()
        .map(Address::from)
        .collect())
    }

    pub fn virt_read_addr64(&mut self, addr: Address) -> Result<Address> {
        Ok(addr!(self.virt_read_u64(addr)?))
    }

    pub fn virt_read_vec_addr64(&mut self, addr: Address, count: usize) -> Result<Vec<Address>> {
        let r = self.virt_read_ret(addr, Length::size_of::<u64>() * count)?;
        Ok(arch_read_vec_type!(
            self.type_arch.instruction_set.byte_order(),
            Length::size_of::<u64>(),
            u64,
            read_u64,
            &r
        )
        .into_iter()
        .map(Address::from)
        .collect())
    }

    pub fn virt_read_u64(&mut self, addr: Address) -> Result<u64> {
        let r = self.virt_read_ret(addr, Length::size_of::<u64>())?;
        Ok(arch_read_type!(
            self.type_arch.instruction_set.byte_order(),
            read_u64,
            &r
        ))
    }

    pub fn virt_read_u32(&mut self, addr: Address) -> Result<u32> {
        let r = self.virt_read_ret(addr, Length::size_of::<u32>())?;
        Ok(arch_read_type!(
            self.type_arch.instruction_set.byte_order(),
            read_u32,
            &r
        ))
    }

    pub fn virt_read_u16(&mut self, addr: Address) -> Result<u16> {
        let r = self.virt_read_ret(addr, Length::size_of::<u16>())?;
        Ok(arch_read_type!(
            self.type_arch.instruction_set.byte_order(),
            read_u16,
            &r
        ))
    }

    pub fn virt_read_u8(&mut self, addr: Address) -> Result<u8> {
        let r = self.virt_read_ret(addr, Length::size_of::<u8>())?;
        Ok(r[0])
    }

    pub fn virt_read_i64(&mut self, addr: Address) -> Result<i64> {
        let r = self.virt_read_ret(addr, Length::size_of::<i64>())?;
        Ok(arch_read_type!(
            self.type_arch.instruction_set.byte_order(),
            read_i64,
            &r
        ))
    }

    pub fn virt_read_i32(&mut self, addr: Address) -> Result<i32> {
        let r = self.virt_read_ret(addr, Length::size_of::<i32>())?;
        Ok(arch_read_type!(
            self.type_arch.instruction_set.byte_order(),
            read_i32,
            &r
        ))
    }

    pub fn virt_read_i16(&mut self, addr: Address) -> Result<i16> {
        let r = self.virt_read_ret(addr, Length::size_of::<i16>())?;
        Ok(arch_read_type!(
            self.type_arch.instruction_set.byte_order(),
            read_i16,
            &r
        ))
    }

    pub fn virt_read_i8(&mut self, addr: Address) -> Result<i8> {
        let r = self.virt_read_ret(addr, Length::size_of::<i8>())?;
        Ok(r[0] as i8)
    }

    pub fn virt_read_f32(&mut self, addr: Address) -> Result<f32> {
        let r = self.virt_read_ret(addr, Length::size_of::<f32>())?;
        Ok(arch_read_type!(
            self.type_arch.instruction_set.byte_order(),
            read_f32,
            &r
        ))
    }

    pub fn virt_read_vec_f32(&mut self, addr: Address, count: usize) -> Result<Vec<f32>> {
        let r = self.virt_read_ret(addr, len!(mem::size_of::<f32>() * count))?;
        Ok(arch_read_vec_type!(
            self.type_arch.instruction_set.byte_order(),
            Length::size_of::<f32>(),
            f32,
            read_f32,
            &r
        ))
    }

    pub fn virt_read_cstr(&mut self, addr: Address, len: usize) -> Result<String> {
        let mut r = self.virt_read_ret(addr, len!(len))?;
        if let Some((n, _)) = r.iter().enumerate().filter(|(_i, c)| **c == 0_u8).nth(0) {
            r.truncate(n);
        }

        let v = CString::new(r)?;
        Ok(String::from(v.to_string_lossy()))
    }

    pub fn virt_read_cstr_ptr(&mut self, addr: Address) -> Result<String> {
        let ptr = self.virt_read_addr(addr)?;
        self.virt_read_cstr(ptr, Length::from_kb(2).as_usize())
    }

    pub fn virt_read_addr_chain(
        &mut self,
        base_addr: Address,
        offsets: Vec<Length>,
    ) -> Result<Address> {
        offsets
            .iter()
            .try_fold(base_addr, |c, &a| self.virt_read_addr(c + a))
    }

    // write helpers
    fn virt_write_addr32(&mut self, addr: Address, data: Address) -> Result<()> {
        self.virt_write_u32(addr, data.as_u32())
    }

    fn virt_write_vec_addr32(&mut self, addr: Address, data: Vec<Address>) -> Result<()> {
        let v = arch_write_vec_type!(
            self.type_arch.instruction_set.byte_order(),
            Length::size_of::<u32>(),
            write_u32,
            data.into_iter().map(Address::as_u32).collect::<Vec<u32>>()
        );
        self.virt_write(addr, &v[..])
    }

    fn virt_write_addr64(&mut self, addr: Address, data: Address) -> Result<()> {
        self.virt_write_u64(addr, data.as_u64())
    }

    fn virt_write_vec_addr64(&mut self, addr: Address, data: Vec<Address>) -> Result<()> {
        let v = arch_write_vec_type!(
            self.type_arch.instruction_set.byte_order(),
            Length::size_of::<u64>(),
            write_u64,
            data.into_iter().map(Address::as_u64).collect::<Vec<u64>>()
        );
        self.virt_write(addr, &v[..])
    }

    fn virt_write_u64(&mut self, addr: Address, data: u64) -> Result<()> {
        let mut v = vec![0_u8; mem::size_of::<u64>()];
        arch_write_type!(
            addr,
            &mut v[..],
            self.type_arch.instruction_set.byte_order(),
            write_u64,
            data
        );
        self.virt_write(addr, &v)
    }

    fn virt_write_u32(&mut self, addr: Address, data: u32) -> Result<()> {
        let mut v = vec![0_u8; mem::size_of::<u32>()];
        arch_write_type!(
            addr,
            &mut v[..],
            self.type_arch.instruction_set.byte_order(),
            write_u32,
            data
        );
        self.virt_write(addr, &v)
    }

    fn virt_write_u16(&mut self, addr: Address, data: u16) -> Result<()> {
        let mut v = vec![0_u8; mem::size_of::<u16>()];
        arch_write_type!(
            addr,
            &mut v[..],
            self.type_arch.instruction_set.byte_order(),
            write_u16,
            data
        );
        self.virt_write(addr, &v)
    }

    fn virt_write_u8(&mut self, addr: Address, data: u8) -> Result<()> {
        let v = vec![data, 1];
        self.virt_write(addr, &v)
    }

    fn virt_write_i64(&mut self, addr: Address, data: i64) -> Result<()> {
        let mut v = vec![0_u8; mem::size_of::<i64>()];
        arch_write_type!(
            addr,
            &mut v[..],
            self.type_arch.instruction_set.byte_order(),
            write_i64,
            data
        );
        self.virt_write(addr, &v)
    }

    fn virt_write_i32(&mut self, addr: Address, data: i32) -> Result<()> {
        let mut v = vec![0_u8; mem::size_of::<i32>()];
        arch_write_type!(
            addr,
            &mut v[..],
            self.type_arch.instruction_set.byte_order(),
            write_i32,
            data
        );
        self.virt_write(addr, &v)
    }

    fn virt_write_i16(&mut self, addr: Address, data: i16) -> Result<()> {
        let mut v = vec![0_u8; mem::size_of::<i16>()];
        arch_write_type!(
            addr,
            &mut v[..],
            self.type_arch.instruction_set.byte_order(),
            write_i16,
            data
        );
        self.virt_write(addr, &v)
    }

    fn virt_write_i8(&mut self, addr: Address, data: i8) -> Result<()> {
        let v = vec![data as u8, 1];
        self.virt_write(addr, &v)
    }

    fn virt_write_f32(&mut self, addr: Address, data: f32) -> Result<()> {
        let mut v = vec![0_u8; mem::size_of::<f32>()];
        arch_write_type!(
            addr,
            &mut v[..],
            self.type_arch.instruction_set.byte_order(),
            write_f32,
            data
        );
        self.virt_write(addr, &v)
    }

    fn virt_write_vec_f32(&mut self, addr: Address, data: Vec<f32>) -> Result<()> {
        let v = arch_write_vec_type!(
            self.type_arch.instruction_set.byte_order(),
            Length::size_of::<f32>(),
            write_f32,
            data
        );
        self.virt_write(addr, &v[..])
    }
}
