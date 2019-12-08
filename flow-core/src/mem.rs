// TODO: custom error + result
use std::io::Result;

use byteorder::{BigEndian, ByteOrder, LittleEndian};

use crate::address::{Address, Length};
use crate::arch::{self, Architecture, InstructionSet};

use std::ffi::{CStr, CString};

pub trait PhysicalRead {
    fn phys_read(&mut self, addr: Address, len: Length) -> Result<Vec<u8>>;
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

pub trait VirtualRead {
    fn virt_read(
        &mut self,
        arch: Architecture,
        dtb: Address,
        addr: Address,
        len: Length,
    ) -> Result<Vec<u8>>;

    fn virt_read_addr(
        &mut self,
        arch: Architecture,
        dtb: Address,
        addr: Address,
    ) -> Result<Address> {
        let r = self.virt_read(arch, dtb, addr, arch.instruction_set.len_addr())?;
        Ok(Address::from(arch_read_type!(
            arch.instruction_set.byte_order(),
            read_u64, // TODO: make this architecture agnostic! (might crash here)
            &r
        )))
    }

    fn virt_read_vec_addr(
        &mut self,
        arch: Architecture,
        dtb: Address,
        addr: Address,
        count: usize,
    ) -> Result<Vec<Address>> {
        let r = self.virt_read(arch, dtb, addr, arch.instruction_set.len_addr() * count)?;
        Ok(arch_read_vec_type!(
            arch.instruction_set.byte_order(),
            arch.instruction_set.len_addr(),
            u64,
            read_u64, // TODO: make this architecture agnostic! (might crash here)
            &r
        )
        .into_iter()
        .map(Address::from)
        .collect())
    }

    fn virt_read_addr32(
        &mut self,
        arch: Architecture,
        dtb: Address,
        addr: Address,
    ) -> Result<Address> {
        let r = self.virt_read(arch, dtb, addr, arch.instruction_set.len_u32())?;
        Ok(Address::from(arch_read_type!(
            arch.instruction_set.byte_order(),
            read_u32,
            &r
        )))
    }

    fn virt_read_vec_addr32(
        &mut self,
        arch: Architecture,
        dtb: Address,
        addr: Address,
        count: usize,
    ) -> Result<Vec<Address>> {
        let r = self.virt_read(arch, dtb, addr, arch.instruction_set.len_u32() * count)?;
        Ok(arch_read_vec_type!(
            arch.instruction_set.byte_order(),
            arch.instruction_set.len_u32(),
            u32,
            read_u32,
            &r
        )
        .into_iter()
        .map(Address::from)
        .collect())
    }

    fn virt_read_addr64(
        &mut self,
        arch: Architecture,
        dtb: Address,
        addr: Address,
    ) -> Result<Address> {
        let r = self.virt_read(arch, dtb, addr, arch.instruction_set.len_u64())?;
        Ok(Address::from(arch_read_type!(
            arch.instruction_set.byte_order(),
            read_u64,
            &r
        )))
    }

    fn virt_read_vec_addr64(
        &mut self,
        arch: Architecture,
        dtb: Address,
        addr: Address,
        count: usize,
    ) -> Result<Vec<Address>> {
        let r = self.virt_read(arch, dtb, addr, arch.instruction_set.len_u64() * count)?;
        Ok(arch_read_vec_type!(
            arch.instruction_set.byte_order(),
            arch.instruction_set.len_u64(),
            u64,
            read_u64,
            &r
        )
        .into_iter()
        .map(Address::from)
        .collect())
    }

    fn virt_read_u64(&mut self, arch: Architecture, dtb: Address, addr: Address) -> Result<u64> {
        let r = self.virt_read(arch, dtb, addr, arch.instruction_set.len_u64())?;
        Ok(arch_read_type!(
            arch.instruction_set.byte_order(),
            read_u64,
            &r
        ))
    }

    fn virt_read_u32(&mut self, arch: Architecture, dtb: Address, addr: Address) -> Result<u32> {
        let r = self.virt_read(arch, dtb, addr, arch.instruction_set.len_u32())?;
        Ok(arch_read_type!(
            arch.instruction_set.byte_order(),
            read_u32,
            &r
        ))
    }

    fn virt_read_u16(&mut self, arch: Architecture, dtb: Address, addr: Address) -> Result<u16> {
        let r = self.virt_read(arch, dtb, addr, arch.instruction_set.len_u16())?;
        Ok(arch_read_type!(
            arch.instruction_set.byte_order(),
            read_u16,
            &r
        ))
    }

    fn virt_read_u8(&mut self, arch: Architecture, dtb: Address, addr: Address) -> Result<u8> {
        let r = self.virt_read(arch, dtb, addr, arch.instruction_set.len_u8())?;
        Ok(r[0])
    }

    fn virt_read_i64(&mut self, arch: Architecture, dtb: Address, addr: Address) -> Result<i64> {
        let r = self.virt_read(arch, dtb, addr, arch.instruction_set.len_i64())?;
        Ok(arch_read_type!(
            arch.instruction_set.byte_order(),
            read_i64,
            &r
        ))
    }

    fn virt_read_i32(&mut self, arch: Architecture, dtb: Address, addr: Address) -> Result<i32> {
        let r = self.virt_read(arch, dtb, addr, arch.instruction_set.len_i32())?;
        Ok(arch_read_type!(
            arch.instruction_set.byte_order(),
            read_i32,
            &r
        ))
    }

    fn virt_read_i16(&mut self, arch: Architecture, dtb: Address, addr: Address) -> Result<i16> {
        let r = self.virt_read(arch, dtb, addr, arch.instruction_set.len_i16())?;
        Ok(arch_read_type!(
            arch.instruction_set.byte_order(),
            read_i16,
            &r
        ))
    }

    fn virt_read_i8(&mut self, arch: Architecture, dtb: Address, addr: Address) -> Result<i8> {
        let r = self.virt_read(arch, dtb, addr, arch.instruction_set.len_i8())?;
        Ok(r[0] as i8)
    }

    fn virt_read_f32(&mut self, arch: Architecture, dtb: Address, addr: Address) -> Result<f32> {
        let r = self.virt_read(arch, dtb, addr, arch.instruction_set.len_f32())?;
        Ok(arch_read_type!(
            arch.instruction_set.byte_order(),
            read_f32,
            &r
        ))
    }

    // TODO: add more vec read helpers
    fn virt_read_vec_f32(
        &mut self,
        arch: Architecture,
        dtb: Address,
        addr: Address,
        count: usize,
    ) -> Result<Vec<f32>> {
        let r = self.virt_read(arch, dtb, addr, arch.instruction_set.len_f32() * count)?;
        Ok(arch_read_vec_type!(
            arch.instruction_set.byte_order(),
            arch.instruction_set.len_f32(),
            f32,
            read_f32,
            &r
        ))
    }

    fn virt_read_cstr(
        &mut self,
        arch: Architecture,
        dtb: Address,
        addr: Address,
        len: usize,
    ) -> Result<String> {
        let mut r = self.virt_read(arch, dtb, addr, len!(len))?;
        match r.iter().enumerate().filter(|(i, c)| **c == 0u8).nth(0) {
            Some((n, _)) => {
                r.truncate(n);
            }
            None => (),
        }

        let v = CString::new(r)?;
        Ok(String::from(v.to_string_lossy()))
    }
}

pub trait PhysicalWrite {
    fn phys_write(&mut self, addr: Address, data: &Vec<u8>) -> Result<Length>;
}

macro_rules! arch_write_type {
    ($byte_order:expr, $func:ident, $buf:expr, $value:expr) => {
        match $byte_order {
            arch::ByteOrder::LittleEndian => LittleEndian::$func($buf, $value),
            arch::ByteOrder::BigEndian => BigEndian::$func($buf, $value),
        }
    };
}

pub trait VirtualWrite {
    fn virt_write(
        &mut self,
        arch: Architecture,
        dtb: Address,
        addr: Address,
        data: &Vec<u8>,
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
