use crate::error::{Error, Result};
use crate::kernel::StartBlock;

use std::convert::TryInto;

use memflow::architecture::x86::x64;
use memflow::types::{size, Address};

// https://github.com/ufrisk/MemProcFS/blob/f2d15cf4fe4f19cfeea3dad52971fae2e491064b/vmm/vmmwininit.c#L560
pub fn find_lowstub(stub: &[u8]) -> Result<StartBlock> {
    Ok(stub
        .chunks_exact(x64::ARCH.page_size())
        .skip(1)
        .filter(|c| {
            (0xffff_ffff_ffff_00ff & u64::from_le_bytes(c[0..8].try_into().unwrap()))
                == 0x0000_0001_0006_00E9
        }) // start bytes
        .filter(|c| {
            (0xffff_f800_0000_0003 & u64::from_le_bytes(c[0x70..0x70 + 8].try_into().unwrap()))
                == 0xffff_f800_0000_0000
        }) // kernel entry
        .find(|c| {
            (0xffff_ff00_0000_0fff & u64::from_le_bytes(c[0xa0..0xa0 + 8].try_into().unwrap())) == 0
        }) // pml4
        .map(|c| StartBlock {
            arch: x64::ARCH,
            kernel_hint: u64::from_le_bytes(c[0x70..0x70 + 8].try_into().unwrap()).into(),
            dtb: u64::from_le_bytes(c[0xa0..0xa0 + 8].try_into().unwrap()).into(),
        })
        .ok_or_else(|| Error::Initialization("unable to find x64 dtb in lowstub < 1M"))?)
}

fn find_pt(addr: Address, mem: &[u8]) -> Option<Address> {
    // TODO: global define / config setting
    let max_mem = size::gb(512) as u64;

    let pte = u64::from_le_bytes(mem[0..8].try_into().unwrap());

    if (pte & 0x0000_0000_0000_0087) != 0x7 || (pte & 0x0000_ffff_ffff_f000) > max_mem {
        return None;
    }

    // Second half must have a self ref entry
    // This is usually enough to filter wrong data out
    mem[0x800..]
        .chunks(8)
        .map(|c| u64::from_le_bytes(c.try_into().unwrap()))
        .find(|a| (a ^ 0x0000_0000_0000_0063) & !(1u64 << 63) == addr.as_u64())?;

    // A page table does need to have some entries, right? Particularly, kernel-side page table
    // entries must be marked as such
    mem[0x800..]
        .chunks(8)
        .map(|c| u64::from_le_bytes(c.try_into().unwrap()))
        .filter(|a| (a & 0xff) == 0x63)
        .nth(5)?;

    Some(addr)
}

pub fn find(mem: &[u8]) -> Result<StartBlock> {
    mem.chunks_exact(x64::ARCH.page_size())
        .enumerate()
        .filter_map(|(i, c)| find_pt((i * x64::ARCH.page_size()).into(), c))
        .map(|addr| StartBlock {
            arch: x64::ARCH,
            kernel_hint: 0.into(),
            dtb: addr,
        })
        .next()
        .ok_or_else(|| Error::Initialization("unable to find x64 dtb in lowstub < 16M"))
}
