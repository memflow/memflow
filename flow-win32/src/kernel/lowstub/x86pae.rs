use crate::error::{Error, Result};
use crate::kernel::StartBlock;

use byteorder::{ByteOrder, LittleEndian};

use flow_core::architecture::{self, Architecture};
use flow_core::types::Address;

fn _find(addr: Address, mem: &[u8]) -> Option<()> {
    for (i, chunk) in mem.to_vec().chunks_exact(8).enumerate() {
        if i < 4 && LittleEndian::read_u64(chunk) == addr.as_u64() + ((i as u64 * 8) << 9) + 0x1001
        {
            return None;
        } else if i >= 4 && LittleEndian::read_u64(chunk) != 0 {
            return None;
        }
    }
    Some(())
}

pub fn find(mem: &[u8]) -> Result<StartBlock> {
    mem.chunks_exact(architecture::x86_pae::page_size().as_usize())
        .enumerate()
        .map(|(i, c)| {
            (
                Address::from(architecture::x86::page_size().as_u64() * i as u64),
                c,
            )
        })
        .find(|(a, c)| _find(a.clone(), c).is_some())
        .ok_or_else(|| Error::new("unable to find x64_pae dtb in lowstub < 16M"))
        .and_then(|(a, _)| {
            Ok(StartBlock {
                arch: Architecture::X86Pae,
                va: Address::from(0),
                dtb: a,
            })
        })
}
