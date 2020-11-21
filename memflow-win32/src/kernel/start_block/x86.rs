use crate::error::{Error, Result};
use crate::kernel::StartBlock;

use std::convert::TryInto;

use memflow::architecture::x86::x32;
use memflow::iter::PageChunks;
use memflow::types::Address;

fn check_page(base: Address, mem: &[u8]) -> bool {
    if mem[0] != 0x67 {
        return false;
    }

    let dword = u32::from_le_bytes(mem[0xc00..0xc00 + 4].try_into().unwrap());
    if (dword & 0xffff_f003) != (base.as_u32() + 0x3) {
        return false;
    }

    matches!(mem
        .iter()
        .step_by(4)
        .skip(0x200)
        .filter(|&&x| x == 0x63 || x == 0xe3)
        .count(), x if x > 16)
}

pub fn find(mem: &[u8]) -> Result<StartBlock> {
    mem.page_chunks(Address::from(0), x32::ARCH.page_size())
        .find(|(a, c)| check_page(*a, c))
        .map(|(a, _)| StartBlock {
            arch: x32::ARCH,
            kernel_hint: 0.into(),
            dtb: a,
        })
        .ok_or_else(|| Error::Initialization("unable to find x86 dtb in lowstub < 16M"))
}
