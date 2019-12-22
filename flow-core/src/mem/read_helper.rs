use crate::error::Result;

use std::mem;
use std::ptr::copy_nonoverlapping;

use byteorder::{BigEndian, ByteOrder, LittleEndian};

use crate::address::{Address, Length};
use crate::arch::{self, Architecture, ArchitectureTrait};
use crate::mem::*;

use std::ffi::CString;

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

// VirtualReadHelper + ArchitectureTrait will enable all read helpers
pub trait VirtualReadHelper {
    fn virt_read(&mut self, addr: Address, len: Length) -> Result<Vec<u8>>;
}

// VirtualReader will wrap all helper methods
pub struct VirtualReader<'a, T: VirtualRead> {
    reader: &'a mut T,
    sys_arch: Architecture,
    type_arch: Architecture,
    dtb: Address,
}

impl<'a, T: VirtualRead> VirtualReader<'a, T> {
    pub fn with(reader: &'a mut T, sys_arch: Architecture, dtb: Address) -> Self {
        Self {
            reader,
            sys_arch,
            type_arch: sys_arch,
            dtb,
        }
    }

    pub fn with_type_arch(
        reader: &'a mut T,
        sys_arch: Architecture,
        type_arch: Architecture,
        dtb: Address,
    ) -> Self {
        Self {
            reader,
            sys_arch,
            type_arch,
            dtb,
        }
    }
}

impl<'a, T: VirtualRead> ArchitectureTrait for VirtualReader<'a, T> {
    fn arch(&mut self) -> Result<Architecture> {
        Ok(self.sys_arch)
    }
}

impl<'a, T: VirtualRead> TypeArchitectureTrait for VirtualReader<'a, T> {
    fn type_arch(&mut self) -> Result<Architecture> {
        Ok(self.type_arch)
    }
}

impl<'a, T: VirtualRead> VirtualReadHelper for VirtualReader<'a, T> {
    fn virt_read(&mut self, addr: Address, len: Length) -> Result<Vec<u8>> {
        self.reader.virt_read(self.sys_arch, self.dtb, addr, len)
    }
}

pub trait VirtualReadHelperFuncs {
    unsafe fn virt_read_raw<U>(&mut self, addr: Address) -> Result<U>;

    fn virt_read_addr(&mut self, addr: Address) -> Result<Address>;

    fn virt_read_vec_addr(&mut self, addr: Address, count: usize) -> Result<Vec<Address>>;

    fn virt_read_addr32(&mut self, addr: Address) -> Result<Address>;

    fn virt_read_vec_addr32(&mut self, addr: Address, count: usize) -> Result<Vec<Address>>;

    fn virt_read_addr64(&mut self, addr: Address) -> Result<Address>;

    fn virt_read_vec_addr64(&mut self, addr: Address, count: usize) -> Result<Vec<Address>>;

    fn virt_read_u64(&mut self, addr: Address) -> Result<u64>;

    fn virt_read_u32(&mut self, addr: Address) -> Result<u32>;

    fn virt_read_u16(&mut self, addr: Address) -> Result<u16>;

    fn virt_read_u8(&mut self, addr: Address) -> Result<u8>;

    fn virt_read_i64(&mut self, addr: Address) -> Result<i64>;

    fn virt_read_i32(&mut self, addr: Address) -> Result<i32>;

    fn virt_read_i16(&mut self, addr: Address) -> Result<i16>;

    fn virt_read_i8(&mut self, addr: Address) -> Result<i8>;

    fn virt_read_f32(&mut self, addr: Address) -> Result<f32>;

    // TODO: add more vec read helpers
    fn virt_read_vec_f32(&mut self, addr: Address, count: usize) -> Result<Vec<f32>>;

    fn virt_read_cstr(&mut self, addr: Address, len: usize) -> Result<String>;
}

impl<T: VirtualReadHelper + ArchitectureTrait + TypeArchitectureTrait> VirtualReadHelperFuncs
    for T
{
    unsafe fn virt_read_raw<U>(&mut self, addr: Address) -> Result<U> {
        let r = self.virt_read(addr, len!(mem::size_of::<U>()))?;
        let mut d = mem::MaybeUninit::<U>::uninit();
        copy_nonoverlapping(r.as_ptr(), d.as_mut_ptr() as *mut u8, r.len());
        Ok(d.assume_init())
    }

    fn virt_read_addr(&mut self, addr: Address) -> Result<Address> {
        let ta = self.type_arch()?;
        let r = self.virt_read(addr, ta.instruction_set.len_addr())?;
        Ok(Address::from(arch_read_type!(
            ta.instruction_set.byte_order(),
            read_u64, // TODO: make this architecture agnostic! (might crash here)
            &r
        )))
    }

    fn virt_read_vec_addr(&mut self, addr: Address, count: usize) -> Result<Vec<Address>> {
        let ta = self.type_arch()?;
        let r = self.virt_read(addr, ta.instruction_set.len_addr() * count)?;
        Ok(arch_read_vec_type!(
            ta.instruction_set.byte_order(),
            ta.instruction_set.len_addr(),
            u64,
            read_u64, // TODO: make this architecture agnostic! (might crash here)
            &r
        )
        .into_iter()
        .map(Address::from)
        .collect())
    }

    fn virt_read_addr32(&mut self, addr: Address) -> Result<Address> {
        Ok(addr!(self.virt_read_u32(addr)?))
    }

    fn virt_read_vec_addr32(&mut self, addr: Address, count: usize) -> Result<Vec<Address>> {
        let ta = self.type_arch()?;
        let r = self.virt_read(addr, ta.instruction_set.len_u32() * count)?;
        Ok(arch_read_vec_type!(
            ta.instruction_set.byte_order(),
            ta.instruction_set.len_u32(),
            u32,
            read_u32,
            &r
        )
        .into_iter()
        .map(Address::from)
        .collect())
    }

    fn virt_read_addr64(&mut self, addr: Address) -> Result<Address> {
        Ok(addr!(self.virt_read_u64(addr)?))
    }

    fn virt_read_vec_addr64(&mut self, addr: Address, count: usize) -> Result<Vec<Address>> {
        let ta = self.type_arch()?;
        let r = self.virt_read(addr, ta.instruction_set.len_u64() * count)?;
        Ok(arch_read_vec_type!(
            ta.instruction_set.byte_order(),
            ta.instruction_set.len_u64(),
            u64,
            read_u64,
            &r
        )
        .into_iter()
        .map(Address::from)
        .collect())
    }

    fn virt_read_u64(&mut self, addr: Address) -> Result<u64> {
        let ta = self.type_arch()?;
        let r = self.virt_read(addr, ta.instruction_set.len_u64())?;
        Ok(arch_read_type!(
            ta.instruction_set.byte_order(),
            read_u64,
            &r
        ))
    }

    fn virt_read_u32(&mut self, addr: Address) -> Result<u32> {
        let ta = self.type_arch()?;
        let r = self.virt_read(addr, ta.instruction_set.len_u32())?;
        Ok(arch_read_type!(
            ta.instruction_set.byte_order(),
            read_u32,
            &r
        ))
    }

    fn virt_read_u16(&mut self, addr: Address) -> Result<u16> {
        let ta = self.type_arch()?;
        let r = self.virt_read(addr, ta.instruction_set.len_u16())?;
        Ok(arch_read_type!(
            ta.instruction_set.byte_order(),
            read_u16,
            &r
        ))
    }

    fn virt_read_u8(&mut self, addr: Address) -> Result<u8> {
        let ta = self.type_arch()?;
        let r = self.virt_read(addr, ta.instruction_set.len_u8())?;
        Ok(r[0])
    }

    fn virt_read_i64(&mut self, addr: Address) -> Result<i64> {
        let ta = self.type_arch()?;
        let r = self.virt_read(addr, ta.instruction_set.len_i64())?;
        Ok(arch_read_type!(
            ta.instruction_set.byte_order(),
            read_i64,
            &r
        ))
    }

    fn virt_read_i32(&mut self, addr: Address) -> Result<i32> {
        let ta = self.type_arch()?;
        let r = self.virt_read(addr, ta.instruction_set.len_i32())?;
        Ok(arch_read_type!(
            ta.instruction_set.byte_order(),
            read_i32,
            &r
        ))
    }

    fn virt_read_i16(&mut self, addr: Address) -> Result<i16> {
        let ta = self.type_arch()?;
        let r = self.virt_read(addr, ta.instruction_set.len_i16())?;
        Ok(arch_read_type!(
            ta.instruction_set.byte_order(),
            read_i16,
            &r
        ))
    }

    fn virt_read_i8(&mut self, addr: Address) -> Result<i8> {
        let ta = self.type_arch()?;
        let r = self.virt_read(addr, ta.instruction_set.len_i8())?;
        Ok(r[0] as i8)
    }

    fn virt_read_f32(&mut self, addr: Address) -> Result<f32> {
        let ta = self.type_arch()?;
        let r = self.virt_read(addr, ta.instruction_set.len_f32())?;
        Ok(arch_read_type!(
            ta.instruction_set.byte_order(),
            read_f32,
            &r
        ))
    }

    // TODO: add more vec read helpers
    fn virt_read_vec_f32(&mut self, addr: Address, count: usize) -> Result<Vec<f32>> {
        let ta = self.type_arch()?;
        let r = self.virt_read(addr, ta.instruction_set.len_f32() * count)?;
        Ok(arch_read_vec_type!(
            ta.instruction_set.byte_order(),
            ta.instruction_set.len_f32(),
            f32,
            read_f32,
            &r
        ))
    }

    fn virt_read_cstr(&mut self, addr: Address, len: usize) -> Result<String> {
        let mut r = self.virt_read(addr, len!(len))?;
        if let Some((n, _)) = r.iter().enumerate().filter(|(_i, c)| **c == 0u8).nth(0) {
            r.truncate(n);
        }

        let v = CString::new(r)?;
        Ok(String::from(v.to_string_lossy()))
    }
}

pub trait VirtualReadHelperChain {
    // TODO: weak chain?
    fn virt_read_addr_chain(&mut self, base_addr: Address, offsets: Vec<Length>)
        -> Result<Address>;
}

// TODO: more error checking?
impl<T: VirtualReadHelperFuncs> VirtualReadHelperChain for T {
    fn virt_read_addr_chain(
        &mut self,
        base_addr: Address,
        offsets: Vec<Length>,
    ) -> Result<Address> {
        offsets
            .iter()
            .try_fold(base_addr, |c, &a| self.virt_read_addr(c + a))
    }
}
