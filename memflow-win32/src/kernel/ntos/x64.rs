use std::prelude::v1::*;

use super::pehelper;
use crate::error::{Error, Result};
use crate::kernel::StartBlock;

use log::{debug, trace};

use memflow::architecture::x86::x64;
use memflow::error::PartialResultExt;
use memflow::iter::PageChunks;
use memflow::mem::VirtualMemory;
use memflow::types::{size, Address};

use dataview::Pod;
use pelite::image::IMAGE_DOS_HEADER;

pub fn find_with_va_hint<T: VirtualMemory>(
    virt_mem: &mut T,
    start_block: &StartBlock,
) -> Result<(Address, usize)> {
    debug!(
        "x64::find_with_va_hint: trying to find ntoskrnl.exe with va hint at {:x}",
        start_block.kernel_hint.as_u64()
    );

    // va was found previously
    let mut va_base = start_block.kernel_hint.as_u64() & !0x0001_ffff;
    while va_base + size::mb(16) as u64 > start_block.kernel_hint.as_u64() {
        trace!("x64::find_with_va_hint: probing at {:x}", va_base);

        match find_with_va(virt_mem, va_base) {
            Ok(a) => {
                let addr = Address::from(a);
                let size_of_image = pehelper::try_get_pe_size(virt_mem, addr)?;
                return Ok((addr, size_of_image));
            }
            Err(e) => trace!("x64::find_with_va_hint: probe error {:?}", e),
        }

        va_base -= size::mb(2) as u64;
    }

    Err(Error::Initialization(
        "x64::find_with_va_hint: unable to locate ntoskrnl.exe via va hint",
    ))
}

fn find_with_va<T: VirtualMemory>(virt_mem: &mut T, va_base: u64) -> Result<u64> {
    let mut buf = vec![0; size::mb(2)];
    virt_mem
        .virt_read_raw_into(Address::from(va_base), &mut buf)
        .data_part()?;

    buf.chunks_exact(x64::ARCH.page_size())
        .enumerate()
        .map(|(i, c)| {
            let view = Pod::as_data_view(&c[..]);
            (i, c, view.copy::<IMAGE_DOS_HEADER>(0)) // TODO: potential endian mismatch
        })
        .filter(|(_, _, p)| p.e_magic == 0x5a4d) // MZ
        .filter(|(_, _, p)| p.e_lfanew <= 0x800)
        .inspect(|(i, _, _)| {
            trace!(
                "x64::find_with_va: found potential header flags at offset {:x}",
                i * x64::ARCH.page_size()
            )
        })
        .find(|(i, _, _)| {
            let probe_addr = Address::from(va_base + (*i as u64) * x64::ARCH.page_size() as u64);
            let name = pehelper::try_get_pe_name(virt_mem, probe_addr).unwrap_or_default();
            name == "ntoskrnl.exe"
        })
        .map(|(i, _, _)| va_base + i as u64 * x64::ARCH.page_size() as u64)
        .ok_or_else(|| Error::Initialization("unable to locate ntoskrnl.exe"))
}

pub fn find<T: VirtualMemory>(
    virt_mem: &mut T,
    start_block: &StartBlock,
) -> Result<(Address, usize)> {
    debug!("x64::find: trying to find ntoskrnl.exe with page map",);

    let page_map = virt_mem.virt_page_map_range(
        size::mb(2),
        (!0u64 - (1u64 << (start_block.arch.address_space_bits() - 1))).into(),
        (!0u64).into(),
    );

    match page_map
        .into_iter()
        .flat_map(|(va, size)| size.page_chunks(va, size::mb(2)))
        .filter(|&(_, size)| size == size::mb(2))
        .filter_map(|(va, _)| find_with_va(virt_mem, va.as_u64()).ok())
        .next()
    {
        Some(a) => {
            let addr = Address::from(a);
            let size_of_image = pehelper::try_get_pe_size(virt_mem, addr)?;
            Ok((addr, size_of_image))
        }
        None => Err(Error::Initialization(
            "x64::find: unable to locate ntoskrnl.exe with a page map",
        )),
    }
}
