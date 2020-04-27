use crate::error::{Error, Result};
use crate::kernel::StartBlock;

use byteorder::{ByteOrder, LittleEndian};

use flow_core::address::{Address, Length};
use flow_core::arch::{self, Architecture};

// see _find_x64
// pa, pb16M + pa
fn _find(mem: &[u8]) -> Option<()> {
    // pa, pb16M + pa

    /*
    for(QWORD i = 0; i < 0x1000; i += 8) {
        if((i < 0x20) && ((*(PQWORD)(pbPage + i) != pa + (i << 9) + 0x1001))) {
            return FALSE;
        } else if((i >= 0x20) && *(PQWORD)(pbPage + i)) {
            return FALSE;
        }
    }
    return TRUE;
    */

    match mem
        .to_vec()
        .chunks_exact(8)
        .skip(3) // >= 0x20
        .filter(|c| c[0] != 0)
        .nth(0)
    {
        Some(_c) => None,
        None => Some(()),
    }
}

pub fn find(mem: &[u8]) -> Result<StartBlock> {
    mem.chunks_exact(arch::x86_pae::page_size().as_usize())
        .position(|c| _find(c).is_some())
        .ok_or_else(|| Error::new("unable to find x64_pae dtb in lowstub < 16M"))
        .and_then(|i| {
            Ok(StartBlock {
                arch: Architecture::X86Pae,
                va: Address::from(0),
                dtb: Address::from((i as u64) * arch::x86_pae::page_size().as_u64()),
            })
        })
}
