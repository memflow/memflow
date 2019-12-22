use crate::error::{Error, Result};

use log::{debug, info, trace, warn};

use byteorder::{ByteOrder, LittleEndian};

use flow_core::address::{Address, Length};
use flow_core::arch::{self, InstructionSet};
use flow_core::mem::VirtualRead;

use goblin::pe::options::ParseOptions;
use goblin::pe::PE;

use crate::kernel::StartBlock;

// TODO: -> Result<WinProcess>
pub fn find<T: VirtualRead>(mem: &mut T, start_block: &StartBlock) -> Result<Address> {
    if start_block.arch.instruction_set == InstructionSet::X64 {
        if !start_block.va.is_null() {
            match find_x64_with_va(mem, start_block) {
                Ok(b) => return Ok(b),
                Err(e) => warn!("{}", e),
            }
        }

        match find_x64(mem) {
            Ok(b) => return Ok(b),
            Err(e) => warn!("{}", e),
        }
    } else {
        match find_x86(mem) {
            Ok(b) => return Ok(b),
            Err(e) => println!("Error: {}", e),
        }
    }

    Err(Error::new("unable to find ntoskrnl.exe"))
}

fn probe_pe_header<T: VirtualRead>(
    mem: &mut T,
    start_block: &StartBlock,
    probe_addr: Address,
) -> Result<String> {
    // TODO: after finding the poolcode we already found the proper ntoskrnl so we probably do not have to parse it any further?
    // try to probe pe header
    let probe_buf = mem
        .virt_read(
            start_block.arch,
            start_block.dtb,
            probe_addr,
            Length::from_mb(32),
        )
        .unwrap();

    let mut pe_opts = ParseOptions::default();
    pe_opts.resolve_rva = false;

    let pe = match PE::parse_with_opts(&probe_buf, &pe_opts) {
        Ok(pe) => {
            trace!("find_x64_with_va: found pe header:\n{:?}", pe);
            pe
        }
        Err(e) => {
            trace!(
                "find_x64_with_va: potential pe header at offset {:x} could not be probed: {:?}",
                probe_addr,
                e
            );
            return Err(Error::from(e));
        }
    };

    info!(
        "find_x64_with_va: found pe header for {}",
        pe.name.unwrap_or_default()
    );
    Ok(pe
        .name
        .ok_or_else(|| Error::new("pe name could not be parsed"))?
        .to_owned())
}

fn find_x64_with_va<T: VirtualRead>(mem: &mut T, start_block: &StartBlock) -> Result<Address> {
    trace!(
        "find_x64_with_va: trying to find ntoskrnl.exe with va hint at {:x}",
        start_block.va.as_u64()
    );

    // va was found previously
    let mut va_base = start_block.va.as_u64() & !0x001f_ffff;
    while va_base + Length::from_mb(32).as_u64() > start_block.va.as_u64() {
        trace!("find_x64_with_va: probing at {:x}", va_base);

        let buf = mem.virt_read(
            start_block.arch,
            start_block.dtb,
            Address::from(va_base),
            Length::from_mb(2),
        )?;
        if buf.is_empty() {
            // TODO: print address as well
            return Err(Error::new(
                "Unable to read memory when scanning for ntoskrnl.exe",
            ));
        }

        let res = buf
            .chunks_exact(arch::x64::page_size().as_usize())
            .enumerate()
            .filter(|(_, c)| LittleEndian::read_u16(&c) == 0x5a4d) // MZ
            .inspect(|(i, _)| {
                trace!(
                    "find_x64_with_va: found potential MZ flag at offset {:x}",
                    i * arch::x64::page_size().as_usize()
                )
            })
            .flat_map(|(i, c)| c.chunks_exact(8).map(move |c| (i, c)))
            .filter(|(_, c)| LittleEndian::read_u64(&c) == 0x4544_4f43_4c4f_4f50) // POOLCODE
            .inspect(|(i, _)| {
                trace!(
                    "find_x64_with_va: found potential POOLCODE flag at offset {:x}",
                    i * arch::x64::page_size().as_usize()
                )
            })
            .filter(|(i, _)| {
                let probe_addr =
                    Address::from(va_base + (*i as u64) * arch::x64::page_size().as_u64());
                let name = probe_pe_header(mem, start_block, probe_addr).unwrap_or_default();
                name == "ntoskrnl.exe"
            })
            .nth(0)
            .ok_or_else(|| {
                Error::new("find_x64_with_va: unable to locate ntoskrnl.exe via va hint")
            })
            .and_then(|(i, _)| Ok(va_base + i as u64 * arch::x64::page_size().as_u64()));
        match res {
            Ok(a) => return Ok(Address::from(a)),
            Err(e) => debug!("{:?}", e),
        }

        va_base -= Length::from_mb(2).as_u64();
    }

    Err(Error::new(
        "find_x64_with_va: unable to locate ntoskrnl.exe via va hint",
    ))
}

fn find_x64<T: VirtualRead>(_mem: &mut T) -> Result<Address> {
    Err(Error::new("find_x64(): not implemented yet"))
}

fn find_x86<T: VirtualRead>(_mem: &mut T) -> Result<Address> {
    Err(Error::new("find_x86(): not implemented yet"))
}
