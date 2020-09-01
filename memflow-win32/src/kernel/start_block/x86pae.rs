use crate::error::{Error, Result};
use crate::kernel::StartBlock;

use std::convert::TryInto;

use memflow::architecture::x86::x32_pae;
use memflow::iter::PageChunks;
use memflow::types::Address;

fn check_page(addr: Address, mem: &[u8]) -> bool {
    for (i, chunk) in mem.to_vec().chunks_exact(8).enumerate() {
        let qword = u64::from_le_bytes(chunk[0..8].try_into().unwrap());
        if (i < 4 && qword != addr.as_u64() + ((i as u64 * 8) << 9) + 0x1001)
            || (i >= 4 && qword != 0)
        {
            return false;
        }
    }
    true
}

pub fn find(mem: &[u8]) -> Result<StartBlock> {
    mem.page_chunks(Address::from(0), x32_pae::ARCH.page_size())
        .find(|(a, c)| check_page(*a, c))
        .map(|(a, _)| StartBlock {
            arch: x32_pae::ARCH,
            kernel_hint: 0.into(),
            dtb: a,
        })
        .ok_or_else(|| Error::Initialization("unable to find x86_pae dtb in lowstub < 16M"))
}
