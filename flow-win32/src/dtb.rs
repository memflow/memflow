// TODO: custom errors
use std::io::{Error, ErrorKind, Result};

use byteorder::{ByteOrder, LittleEndian};

use flow_core::machine::Machine;
use flow_core::cpu::Architecture;

pub struct DTB {
    pub arch: Architecture,
    pub va: u64,
    pub dtb: u64,
}

pub struct Windows {
    dtb: DTB,
}

pub fn find(machine: &mut Machine) -> Result<DTB> {
    // read low 1mb stub
    let low1m = machine.mem.read_physical_memory(0, 0x100000)?;

    // find x64 dtb in low stub < 1M
    match find_x64_lowstub(&low1m) {
        Ok(d) => return Ok(d),
        Err(e) => println!("Error: {}", e),
    }

    // TODO: append instead of read twice?
    // read low 16mb stub
    let low16m = machine.mem.read_physical_memory(0, 0x1000000)?;

    match find_x64(&low16m) {
        Ok(d) => return Ok(d),
        Err(e) => println!("Error: {}", e),
    }

    match find_x86_pae(&low16m) {
        Ok(d) => return Ok(d),
        Err(e) => println!("Error: {}", e),
    }

    match find_x86(&low16m) {
        Ok(d) => return Ok(d),
        Err(e) => println!("Error: {}", e),
    }

    Err(Error::new(ErrorKind::Other, "unable to find dtb"))
}

fn find_x64_lowstub(stub: &Vec<u8>) -> Result<DTB> {
    stub.chunks_exact(0x1000)
        .skip(1)
        .filter(|c| (0xffffffffffff00ff & LittleEndian::read_u64(&c)) == 0x00000001000600E9) // start bytes
        .filter(|c| (0xfffff80000000003 & LittleEndian::read_u64(&c[0x70..])) == 0xfffff80000000000) // kernel entry
        .filter(|c| (0xffffff0000000fff & LittleEndian::read_u64(&c[0xA0..])) == 0) // pml4
        .nth(0)
        .ok_or_else(|| Error::new(ErrorKind::Other, "unable to find x64 dtb in lowstub < 1M"))
        .and_then(|c| {
            Ok(DTB {
                arch: Architecture::X64,
                va: LittleEndian::read_u64(&c[0x70..]),
                dtb: LittleEndian::read_u64(&c[0xA0..]),
            })
        })
}

/*
* Check if a page looks like the Windows Kernel x86 Directory Table Base (DTB)
* in the 32-bit PAE memory mode - i.e. the PDPT of the System process.
* Also please note that this may not be the actual PDPT used by the kernel -
* it may very well rather be the PDPT probably set up by WinLoad and then the
* 'System' process uses another. But it works for auto-detect!
* 1: (4) valid PDPT entries with consecutive physical addresses of the PDPT.
* 2: all zeroes for the rest of the page.
*/
fn _find_x64(mem: &[u8]) -> Option<()> {
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
    None
}

fn find_x64(mem: &Vec<u8>) -> Result<DTB> {
    mem.chunks_exact(0x1000)
        .position(|c| _find_x64(c).is_some())
        .ok_or_else(|| Error::new(ErrorKind::Other, "unable to find x64 dtb in lowstub < 16M"))
        .and_then(|i| {
            Ok(DTB {
                arch: Architecture::X64,
                va: 0,
                dtb: (i as u64) * 0x1000,
            })
        })
}

// see _find_x64
// pa, pb16M + pa
fn _find_x86_pae(mem: &[u8]) -> Option<()> {
    // pa, pb16M + pa

    /*
    match mem.to_vec()
    .chunks_exact(8)
    .take(3) // < 0x20
    .filter(|c| c[0] != pa + (i << 9) + 0x1001)
    .nth(0) {
        Some(_c) => return false,
        None => (),
    }
    */

    match mem
        .to_vec()
        .chunks_exact(8)
        .skip(3) // >= 0x20
        .filter(|c| c[0] != 0)
        .nth(0)
    {
        Some(_c) => return None,
        None => return Some(()),
    }

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
}

fn find_x86_pae(mem: &Vec<u8>) -> Result<DTB> {
    mem.chunks_exact(0x1000)
        .position(|c| _find_x86_pae(c).is_some())
        .ok_or_else(|| {
            Error::new(
                ErrorKind::Other,
                "unable to find x64_pae dtb in lowstub < 16M",
            )
        })
        .and_then(|i| {
            Ok(DTB {
                arch: Architecture::X86Pae,
                va: 0,
                dtb: (i as u64) * 0x1000,
            })
        })
}

/*
* Check if a page looks like the Windows Kernel x86 Directory Table Base (DTB)
* in the 32-bit mode -  i.e. the PD of the System process.
* 1: self-referential entry exists at offset 0xC00
* 2: PDE[0] is a user-mode PDE pointing to a PT.
* 3: a minimum number of supervisor-mode PDEs must exist.
*/
fn _find_x86(mem: &[u8]) -> Option<()> {
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

fn find_x86(mem: &Vec<u8>) -> Result<DTB> {
    mem.chunks_exact(0x1000)
        .position(|c| _find_x86(c).is_some())
        .ok_or_else(|| Error::new(ErrorKind::Other, "unable to find x86 dtb in lowstub < 16M"))
        .and_then(|i| {
            Ok(DTB {
                arch: Architecture::X86,
                va: 0,
                dtb: (i as u64) * 0x1000,
            })
        })
}
