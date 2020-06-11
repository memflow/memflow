use crate::error::{Error, Result};
use crate::kernel::StartBlock;

use byteorder::{ByteOrder, LittleEndian};

use flow_core::architecture::{self, Architecture};
use flow_core::types::Address;

fn _find(base: Address, mem: &[u8]) -> Option<()> {
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

    if (LittleEndian::read_u32(&mem[0xC00..]) & 0xfffff003) != (base.as_u32() + 0x3) {
        return None;
    }
    println!("first check passed");

    None
}

pub fn find(mem: &[u8]) -> Result<StartBlock> {
    mem.chunks_exact(architecture::x86::page_size().as_usize())
        .enumerate()
        .map(|(i, c)| (Address::from(architecture::x86::page_size().as_u64() * i as u64), c))
        .find(|(a, c)| _find(a.clone(), c).is_some())
        .ok_or_else(|| Error::new("unable to find x86 dtb in lowstub < 16M"))
        .and_then(|(a, _)| {
            Ok(StartBlock {
                arch: Architecture::X86,
                va: Address::from(0),
                dtb: a,
            })
        })
}
