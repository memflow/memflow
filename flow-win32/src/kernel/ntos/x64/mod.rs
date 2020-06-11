mod pe;
use pe::*;

use crate::error::{Error, Result};
use crate::kernel::StartBlock;

use byteorder::{ByteOrder, LittleEndian};
use log::debug;

use flow_core::architecture;
use flow_core::mem::VirtualMemory;
use flow_core::types::{Address, Length};

use dataview::Pod;
use pelite::image::IMAGE_DOS_HEADER;

pub fn find_with_va<T: VirtualMemory + ?Sized>(
    virt_mem: &mut T,
    start_block: &StartBlock,
) -> Result<(Address, Length)> {
    debug!(
        "x64::find_with_va: trying to find ntoskrnl.exe with va hint at {:x}",
        start_block.va.as_u64()
    );

    // va was found previously
    let mut va_base = start_block.va.as_u64() & !0x0001_ffff;
    while va_base + Length::from_mb(16).as_u64() > start_block.va.as_u64() {
        debug!("find_x64_with_va: probing at {:x}", va_base);

        let mut buf = vec![0; Length::from_mb(2).as_usize()];
        virt_mem.virt_read_raw_into(Address::from(va_base), &mut buf)?;

        let res = buf
            .chunks_exact(architecture::x64::page_size().as_usize())
            .enumerate()
            .map(|(i, c)| {
                let view = Pod::as_data_view(&c[..]);
                (i, c, view.copy::<IMAGE_DOS_HEADER>(0)) // TODO: potential endian mismatch
            })
            .filter(|(_, _, p)| p.e_magic == 0x5a4d) // MZ
            .filter(|(_, _, p)| p.e_lfanew <= 0x800)
            .inspect(|(i, _, _)| {
                debug!(
                    "find_x64_with_va: found potential header flags at offset {:x}",
                    i * architecture::x64::page_size().as_usize()
                )
            })
            .flat_map(|(i, c, p)| c.chunks_exact(8).map(move |c| (i, c, p)))
            .filter(|(_, c, _)| LittleEndian::read_u64(&c) == 0x4544_4f43_4c4f_4f50) // POOLCODE
            .inspect(|(i, _, _)| {
                debug!(
                    "find_x64_with_va: found potential POOLCODE flag at offset {:x}",
                    i * architecture::x64::page_size().as_usize()
                )
            })
            .find(|(i, _, _)| {
                let probe_addr = Address::from(
                    va_base + (*i as u64) * architecture::x64::page_size().as_u64(),
                );
                let name = try_get_pe_name(virt_mem, probe_addr).unwrap_or_default();
                name == "ntoskrnl.exe"
            })
            .ok_or_else(|| {
                Error::new("find_x64_with_va: unable to locate ntoskrnl.exe via va hint")
            })
            .and_then(|(i, _, _)| {
                Ok(va_base + i as u64 * architecture::x64::page_size().as_u64())
            });

        match res {
            Ok(a) => {
                let addr = Address::from(a);
                let size_of_image = try_get_pe_size(virt_mem, addr)?;
                return Ok((addr, size_of_image));
            }
            Err(e) => {
                debug!("{:?}", e);
            }
        }

        va_base -= Length::from_mb(2).as_u64();
    }

    Err(Error::new(
        "find_x64_with_va: unable to locate ntoskrnl.exe via va hint",
    ))
}

pub fn find<T: VirtualMemory + ?Sized>(_mem: &mut T) -> Result<(Address, Length)> {
    Err(Error::new("find_x64(): not implemented yet"))
}