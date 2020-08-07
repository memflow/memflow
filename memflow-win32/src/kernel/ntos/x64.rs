use std::prelude::v1::*;

use super::pehelper;
use crate::error::{Error, Result};
use crate::kernel::StartBlock;

use log::debug;

use memflow_core::architecture;
use memflow_core::error::PartialResultExt;
use memflow_core::mem::VirtualMemory;
use memflow_core::types::{size, Address};

use dataview::Pod;
use pelite::image::IMAGE_DOS_HEADER;

pub fn find_with_va<T: VirtualMemory>(
    virt_mem: &mut T,
    start_block: &StartBlock,
) -> Result<(Address, usize)> {
    debug!(
        "x64::find_with_va: trying to find ntoskrnl.exe with va hint at {:x}",
        start_block.kernel_hint.as_u64()
    );

    // va was found previously
    let mut va_base = start_block.kernel_hint.as_u64() & !0x0001_ffff;
    while va_base + size::mb(16) as u64 > start_block.kernel_hint.as_u64() {
        debug!("x64::find_with_va: probing at {:x}", va_base);

        let mut buf = vec![0; size::mb(2)];
        virt_mem
            .virt_read_raw_into(Address::from(va_base), &mut buf)
            .data_part()?;

        let res = buf
            .chunks_exact(architecture::x64::page_size())
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
                    i * architecture::x64::page_size()
                )
            })
            .find(|(i, _, _)| {
                let probe_addr =
                    Address::from(va_base + (*i as u64) * architecture::x64::page_size() as u64);
                let name = pehelper::try_get_pe_name(virt_mem, probe_addr).unwrap_or_default();
                name == "ntoskrnl.exe"
            })
            .map(|(i, _, _)| va_base + i as u64 * architecture::x64::page_size() as u64)
            .ok_or_else(|| {
                Error::Initialization("find_x64_with_va: unable to locate ntoskrnl.exe via va hint")
            });

        match res {
            Ok(a) => {
                // TODO: unify pe name + size
                let addr = Address::from(a);
                let size_of_image = pehelper::try_get_pe_size(virt_mem, addr)?;
                return Ok((addr, size_of_image));
            }
            Err(e) => debug!("{:?}", e),
        }

        va_base -= size::mb(2) as u64;
    }

    Err(Error::Initialization(
        "find_x64_with_va: unable to locate ntoskrnl.exe via va hint",
    ))
}

pub fn find<T: VirtualMemory>(_mem: &mut T) -> Result<(Address, usize)> {
    Err(Error::Initialization("find_x64(): not implemented yet"))
}
