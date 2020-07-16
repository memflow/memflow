use crate::error::{Error, Result};
use crate::kernel::StartBlock;

use log::info;

use byteorder::{ByteOrder, LittleEndian};

use memflow_core::architecture::{self, Architecture};
use memflow_core::types::size;

// https://github.com/ufrisk/MemProcFS/blob/f2d15cf4fe4f19cfeea3dad52971fae2e491064b/vmm/vmmwininit.c#L560
pub fn find_lowstub(stub: &[u8]) -> Result<StartBlock> {
    stub.chunks_exact(architecture::x64::page_size())
        .skip(1)
        .filter(|c| (0xffff_ffff_ffff_00ff & LittleEndian::read_u64(&c)) == 0x0000_0001_0006_00E9) // start bytes
        .filter(|c| {
            (0xffff_f800_0000_0003 & LittleEndian::read_u64(&c[0x70..])) == 0xffff_f800_0000_0000
        }) // kernel entry
        .find(|c| (0xffff_ff00_0000_0fff & LittleEndian::read_u64(&c[0xA0..])) == 0) // pml4
        .ok_or_else(|| Error::Initialization("unable to find x64 dtb in lowstub < 1M"))
        .and_then(|c| {
            Ok(StartBlock {
                arch: Architecture::X64,
                kernel_hint: LittleEndian::read_u64(&c[0x70..]).into(),
                dtb: LittleEndian::read_u64(&c[0xA0..]).into(),
            })
        })
}

fn _find(mem: &[u8]) -> Option<()> {
    /*
    DWORD c, i;
    BOOL fSelfRef = FALSE;
    QWORD pte, paMax;
    paMax = ctxMain->dev.paMax;
    // check for user-mode page table with PDPT below max physical address and not NX.
    pte = *(PQWORD)pbPage;
    if(((pte & 0x0000000000000087) != 0x07) || ((pte & 0x0000fffffffff000) > paMax)) { return FALSE; }
    for(c = 0, i = 0x800; i < 0x1000; i += 8) { // minimum number of supervisor entries above 0x800
        pte = *(PQWORD)(pbPage + i);
        // check for user-mode page table with PDPT below max physical address and not NX.
        if(((pte & 0x8000ff0000000087) == 0x03) && ((pte & 0x0000fffffffff000) < paMax)) { c++; }
        // check for self-referential entry
        if((*(PQWORD)(pbPage + i) & 0x0000fffffffff083) == pa + 0x03) { fSelfRef = TRUE; }
    }
    return fSelfRef && (c >= 6);
    */

    // TODO: global define / config setting
    let max_mem = size::gb(16) as u64;

    let pte = LittleEndian::read_u64(&mem);
    if (pte & 0x0000_0000_0000_0087) == 0x3 || (pte & 0x0000_ffff_ffff_f000) > max_mem {
        return None;
    }

    info!("found potential entry");

    None
}

pub fn find(mem: &[u8]) -> Result<StartBlock> {
    mem.chunks_exact(architecture::x64::page_size())
        .position(|c| _find(c).is_some())
        .ok_or_else(|| Error::Initialization("unable to find x64 dtb in lowstub < 16M"))
        .and_then(|i| {
            Ok(StartBlock {
                arch: Architecture::X64,
                kernel_hint: 0.into(),
                dtb: ((i as u64) * architecture::x64::page_size() as u64).into(),
            })
        })
}
