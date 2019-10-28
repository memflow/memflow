// TODO: custom error + result
use std::io::Result;

use byteorder::{BigEndian, ByteOrder, LittleEndian};

use address::{Address, Length};
use arch::{self, Architecture, InstructionSet};

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
            read_u64,
            &r
        )))
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

    fn virt_read_f32(&mut self, arch: Architecture, dtb: Address, addr: Address) -> Result<f32> {
        let r = self.virt_read(arch, dtb, addr, arch.instruction_set.len_f32())?;
        Ok(arch_read_type!(
            arch.instruction_set.byte_order(),
            read_f32,
            &r
        ))
    }
}

pub trait PhysicalWrite {
    fn phys_write(&mut self, addr: Address, data: &Vec<u8>) -> Result<Length>;
}

pub trait VirtualWrite {
    fn virt_write(
        &mut self,
        arch: Architecture,
        dtb: Address,
        addr: Address,
        data: &Vec<u8>,
    ) -> Result<Length>;
}
