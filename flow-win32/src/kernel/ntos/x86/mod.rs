mod pe;
use pe::*;

use crate::error::{Error, Result};
use crate::kernel::StartBlock;

use byteorder::{ByteOrder, LittleEndian};
use log::debug;

use flow_core::mem::VirtualMemory;
use flow_core::types::{Address, Length};

use dataview::Pod;
use pelite::image::IMAGE_DOS_HEADER;

const LENGTH_64MB: Length = Length::from_mb(64);
const LENGTH_8MB: Length = Length::from_mb(8);
const LENGTH_4KB: Length = Length::from_kb(4);

// https://github.com/ufrisk/MemProcFS/blob/f2d15cf4fe4f19cfeea3dad52971fae2e491064b/vmm/vmmwininit.c#L410
pub fn find<T: VirtualMemory + ?Sized>(virt_mem: &mut T, start_block: &StartBlock) -> Result<(Address, Length)> {
    debug!("x86::find: trying to find ntoskrnl.exe");

    for base_addr in (0..LENGTH_64MB.as_u64()).step_by(LENGTH_8MB.as_usize()) {
        // search in each page in the first 8mb chunks in the first 64mb of virtual memory
        let mem = virt_mem.virt_read_raw(Address::from(base_addr), LENGTH_8MB)?;
        for addr in (base_addr..LENGTH_8MB.as_u64()).step_by(LENGTH_4KB.as_usize()) {
            // TODO: potential endian mismatch in pod
            let view = Pod::as_data_view(&mem[addr as usize..]);

            // check for dos header signature (MZ) // TODO: create global
            if view.read::<IMAGE_DOS_HEADER>(0).e_magic != 0x5a4d {
                continue;
            }

            if view.read::<IMAGE_DOS_HEADER>(0).e_lfanew > 0x800 {
                continue;
            }

            for offset in (0..0x800).step_by(8) {
                if LittleEndian::read_u64(&mem[(addr + offset) as usize..]) == 0x4544_4f43_4c4f_4f50 {
                    if let Ok(name) = try_get_pe_name(virt_mem, Address::from(addr + offset)) {
                        if name == "ntoskrnl.exe" {
                            println!("ntoskrnl found");
                        } else {
                            // continue 'for addr in ...'
                        }
                    } else {
                        // vaNtosTry = 0x80000000 + p;
                        // continue 'for addr in ...'
                    }
                }
            }
        }
    }

    // return vaNtosTry;

    Err(Error::new("find_x86(): not implemented yet"))
}
