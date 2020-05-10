use crate::error::{Error, Result};
use crate::kernel::StartBlock;

use flow_core::arch::{self, Architecture};
use flow_core::types::Address;

fn _find(_mem: &[u8]) -> Option<()> {
    /*
    DWORD c, i;
    if((*(PDWORD)(pbPage + 0xc00) & 0xfffff003) != pa + 0x03) { return FALSE; } // self-referential entry exists
    if(*pbPage != 0x67) { return FALSE; }  // user-mode page table exists at 1st PTE (index 0)
    for(c = 0, i = 0x800; i < 0x1000; i += 4) { // minimum number of supervisor entries above 0x800
        if((*(pbPage + i) == 0x63) || (*(pbPage + i) == 0xe3)) { c++; }
        if(c > 16) { return TRUE; }
    }
    return FALSE;
    */
    None
}

pub fn find(mem: &[u8]) -> Result<StartBlock> {
    mem.chunks_exact(arch::x86::page_size().as_usize())
        .position(|c| _find(c).is_some())
        .ok_or_else(|| Error::new("unable to find x86 dtb in lowstub < 16M"))
        .and_then(|i| {
            Ok(StartBlock {
                arch: Architecture::X86,
                va: Address::from(0),
                dtb: Address::from((i as u64) * arch::x86::page_size().as_u64()),
            })
        })
}
