// TODO: custom error + result
use std::io::Result;
use std::mem;
use std::ptr::copy_nonoverlapping;

use byteorder::{BigEndian, ByteOrder, LittleEndian};

use crate::address::{Address, Length};
use crate::arch::{self, Arch, Architecture};

use std::ffi::CString;

/*
    mem.virt_read(arch, dtb).as_bytes(0x1000, 5)
    mem.virt_read(arch, dtb).raw<T>(0x1000)
    mem.virt_read(arch, dtb).addr(0x1000)
    mem.virt_read(arch, dtb).u64(0x1000)
*/

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

// virt_read().as_u64() <<<
pub trait MemoryReader<'a> {
    fn as_bytes(&'a mut self, addr: Address, len: Length) -> Result<Vec<u8>>;
}

pub trait MemoryReaderHelper {
    unsafe fn raw<T>(
        &mut self,
        addr: Address,
    ) -> Result<T>;

    /*
    fn addr(
        &mut self,
        addr: Address,
    ) -> Result<Address> {
    }

    fn vec_addr(
        &mut self,
        addr: Address,
        count: usize,
    ) -> Result<Vec<Address>> {
    }*/

    fn addr32(
        &mut self,
        addr: Address,
    ) -> Result<Address>;
    fn vec_addr32(
        &mut self,
        addr: Address,
        count: usize,
    ) -> Result<Vec<Address>>;
    fn addr64(
        &mut self,
        addr: Address,
    ) -> Result<Address>;
    fn vec_addr64(
        &mut self,
        addr: Address,
        count: usize,
    ) -> Result<Vec<Address>>;
    fn u64(&mut self, addr: Address) -> Result<u64>;
    fn u32(&mut self, addr: Address) -> Result<u32>;
    fn u16(&mut self, addr: Address) -> Result<u16>;
    fn u8(&mut self, addr: Address) -> Result<u8>;
    fn i64(&mut self, addr: Address) -> Result<i64>;
    fn i32(&mut self, addr: Address) -> Result<i32>;
    fn i16(&mut self, addr: Address) -> Result<i16>;
    fn i8(&mut self, addr: Address) -> Result<i8>;
    fn f32(&mut self, addr: Address) -> Result<f32>;
    fn vec_f32(
        &mut self,
        addr: Address,
        count: usize,
    ) -> Result<Vec<f32>>;
    fn cstr(
        &mut self,
        addr: Address,
        len: usize,
    ) -> Result<String>;
}

impl<'a, R: MemoryReader<'a> + Arch> MemoryReaderHelper for R {
    /// # Safety
    /// 
    /// raw does not check for endianess or other constraints and might therefor crash
    unsafe fn raw<T>(
        &mut self,
        addr: Address,
    ) -> Result<T> {
        let r = self.as_bytes(addr, len!(mem::size_of::<T>()))?;
        let mut d = mem::MaybeUninit::<T>::uninit();
        copy_nonoverlapping(r.as_ptr(), d.as_mut_ptr() as *mut u8, r.len());
        Ok(d.assume_init())
    }

    /*
    fn addr(
        &mut self,
        addr: Address,
    ) -> Result<Address> {
        let r = self.as_bytes(addr, arch.instruction_set.len_addr())?;
        Ok(Address::from(arch_read_type!(
            arch.instruction_set.byte_order(),
            read_u64, // TODO: make this architecture agnostic! (might crash here)
            &r
        )))
    }

    fn vec_addr(
        &mut self,
        addr: Address,
        count: usize,
    ) -> Result<Vec<Address>> {
        let r = self.as_bytes(addr, arch.instruction_set.len_addr() * count)?;
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
    }*/

    fn addr32(
        &mut self,
        addr: Address,
    ) -> Result<Address> {
        let arch = self.arch();
        let r = self.as_bytes(addr, arch.instruction_set.len_u32())?;
        Ok(Address::from(arch_read_type!(
            arch.instruction_set.byte_order(),
            read_u32,
            &r
        )))
    }

    fn vec_addr32(
        &mut self,
        addr: Address,
        count: usize,
    ) -> Result<Vec<Address>> {
        let arch = self.arch();
        let r = self.as_bytes(addr, arch.instruction_set.len_u32() * count)?;
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

    fn addr64(
        &mut self,
        addr: Address,
    ) -> Result<Address> {
        let arch = self.arch();
        let r = self.as_bytes(addr, arch.instruction_set.len_u64())?;
        Ok(Address::from(arch_read_type!(
            arch.instruction_set.byte_order(),
            read_u64,
            &r
        )))
    }

    fn vec_addr64(
        &mut self,
        addr: Address,
        count: usize,
    ) -> Result<Vec<Address>> {
        let arch = self.arch();
        let r = self.as_bytes(addr, arch.instruction_set.len_u64() * count)?;
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

    fn u64(&mut self, addr: Address) -> Result<u64> {
        let arch = self.arch();
        let r = self.as_bytes(addr, arch.instruction_set.len_u64())?;
        Ok(arch_read_type!(
            arch.instruction_set.byte_order(),
            read_u64,
            &r
        ))
    }

    fn u32(&mut self, addr: Address) -> Result<u32> {
        let arch = self.arch();
        let r = self.as_bytes(addr, arch.instruction_set.len_u32())?;
        Ok(arch_read_type!(
            arch.instruction_set.byte_order(),
            read_u32,
            &r
        ))
    }

    fn u16(&mut self, addr: Address) -> Result<u16> {
        let arch = self.arch();
        let r = self.as_bytes(addr, arch.instruction_set.len_u16())?;
        Ok(arch_read_type!(
            arch.instruction_set.byte_order(),
            read_u16,
            &r
        ))
    }

    fn u8(&mut self, addr: Address) -> Result<u8> {
        let arch = self.arch();
        let r = self.as_bytes(addr, arch.instruction_set.len_u8())?;
        Ok(r[0])
    }

    fn i64(&mut self, addr: Address) -> Result<i64> {
        let arch = self.arch();
        let r = self.as_bytes(addr, arch.instruction_set.len_i64())?;
        Ok(arch_read_type!(
            arch.instruction_set.byte_order(),
            read_i64,
            &r
        ))
    }

    fn i32(&mut self, addr: Address) -> Result<i32> {
        let arch = self.arch();
        let r = self.as_bytes(addr, arch.instruction_set.len_i32())?;
        Ok(arch_read_type!(
            arch.instruction_set.byte_order(),
            read_i32,
            &r
        ))
    }

    fn i16(&mut self, addr: Address) -> Result<i16> {
        let arch = self.arch();
        let r = self.as_bytes(addr, arch.instruction_set.len_i16())?;
        Ok(arch_read_type!(
            arch.instruction_set.byte_order(),
            read_i16,
            &r
        ))
    }

    fn i8(&mut self, addr: Address) -> Result<i8> {
        let arch = self.arch();
        let r = self.as_bytes(addr, arch.instruction_set.len_i8())?;
        Ok(r[0] as i8)
    }

    fn f32(&mut self, addr: Address) -> Result<f32> {
        let arch = self.arch();
        let r = self.as_bytes(addr, arch.instruction_set.len_f32())?;
        Ok(arch_read_type!(
            arch.instruction_set.byte_order(),
            read_f32,
            &r
        ))
    }

    // TODO: add more vec read helpers
    fn vec_f32(
        &mut self,
        addr: Address,
        count: usize,
    ) -> Result<Vec<f32>> {
        let arch = self.arch();
        let r = self.as_bytes(addr, arch.instruction_set.len_f32() * count)?;
        Ok(arch_read_vec_type!(
            arch.instruction_set.byte_order(),
            arch.instruction_set.len_f32(),
            f32,
            read_f32,
            &r
        ))
    }

    fn cstr(
        &mut self,
        addr: Address,
        len: usize,
    ) -> Result<String> {
        let mut r = self.as_bytes(addr, len!(len))?;
        if let Some((n, _)) = r.iter().enumerate().filter(|(_i, c)| **c == 0_u8).nth(0) {
            r.truncate(n);
        }

        let v = CString::new(r)?;
        Ok(String::from(v.to_string_lossy()))
    }
}
