use crate::error::{Error, Result};

use log::{debug, info, trace, warn};

use byteorder::{ByteOrder, LittleEndian};

use flow_core::address::{Address, Length};
use flow_core::arch::{self, Architecture};
use flow_core::mem::AccessVirtualMemory;

use crate::kernel::StartBlock;

use pelite::{self, image::GUID, pe64::debug::CodeView, PeView};

use uuid::{self, Uuid};

// TODO: -> Result<WinProcess>
pub fn find<T: AccessVirtualMemory>(
    mem: &mut T,
    start_block: &StartBlock,
) -> Result<(Address, Length)> {
    if start_block.arch.bits() == 64 {
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
    } else if start_block.arch.bits() == 32 {
        match find_x86(mem) {
            Ok(b) => return Ok(b),
            Err(e) => println!("Error: {}", e),
        }
    }

    Err(Error::new("unable to find ntoskrnl.exe"))
}

pub fn try_fetch_pe_size<T: AccessVirtualMemory>(
    mem: &mut T,
    start_block: &StartBlock,
    addr: Address,
) -> Result<Length> {
    // try to probe pe header
    let mut probe_buf = vec![0; Length::from_kb(4).as_usize()];
    mem.virt_read_raw_into(start_block.arch, start_block.dtb, addr, &mut probe_buf)?;

    let pe_probe = match PeView::from_bytes(&probe_buf) {
        Ok(pe) => {
            trace!("try_fetch_pe_size: found pe header.");
            pe
        }
        Err(e) => {
            trace!(
                "try_fetch_pe_size: potential pe header at offset {:x} could not be probed: {:?}",
                addr,
                e
            );
            return Err(Error::from(e));
        }
    };

    let opt_header = pe_probe.optional_header();
    let size_of_image = match opt_header {
        pelite::Wrap::T32(opt32) => opt32.SizeOfImage,
        pelite::Wrap::T64(opt64) => opt64.SizeOfImage,
    };
    info!(
        "try_fetch_pe_size: found pe header for image with a size of {} bytes.",
        size_of_image
    );
    Ok(Length::from(size_of_image))
}

pub fn try_fetch_pe_header<T: AccessVirtualMemory>(
    mem: &mut T,
    start_block: &StartBlock,
    addr: Address,
) -> Result<Vec<u8>> {
    let size_of_image = try_fetch_pe_size(mem, start_block, addr)?;
    let mut buf = vec![0; size_of_image.as_usize()];
    mem.virt_read_raw_into(start_block.arch, start_block.dtb, addr, &mut buf)?;
    Ok(buf)
}

// TODO: store pe size in windows struct so we can reference it later
fn probe_pe_header<T: AccessVirtualMemory>(
    mem: &mut T,
    start_block: &StartBlock,
    probe_addr: Address,
) -> Result<String> {
    // try to probe pe header
    let pe_buf = try_fetch_pe_header(mem, start_block, probe_addr)?;

    let pe = match PeView::from_bytes(&pe_buf) {
        Ok(pe) => pe,
        Err(e) => {
            trace!(
                    "probe_pe_header: potential pe header at offset {:x} could not be fully probed: {:?}",
                    probe_addr,
                    e
                );
            return Err(Error::from(e));
        }
    };

    let name = pe.exports()?.dll_name()?.to_str()?;
    info!("probe_pe_header: found pe header for {}", name);
    Ok(name.to_string())
}

fn find_x64_with_va<T: AccessVirtualMemory>(
    mem: &mut T,
    start_block: &StartBlock,
) -> Result<(Address, Length)> {
    trace!(
        "find_x64_with_va: trying to find ntoskrnl.exe with va hint at {:x}",
        start_block.va.as_u64()
    );

    // va was found previously
    let mut va_base = start_block.va.as_u64() & !0x001f_ffff;
    while va_base + Length::from_mb(32).as_u64() > start_block.va.as_u64() {
        trace!("find_x64_with_va: probing at {:x}", va_base);

        let mut buf = vec![0; Length::from_mb(2).as_usize()];
        mem.virt_read_raw_into(
            start_block.arch,
            start_block.dtb,
            Address::from(va_base),
            &mut buf,
        )?;

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
            Ok(a) => {
                let addr = Address::from(a);
                let size_of_image = try_fetch_pe_size(mem, start_block, addr)?;
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

fn find_x64<T: AccessVirtualMemory>(_mem: &mut T) -> Result<(Address, Length)> {
    Err(Error::new("find_x64(): not implemented yet"))
}

fn find_x86<T: AccessVirtualMemory>(_mem: &mut T) -> Result<(Address, Length)> {
    Err(Error::new("find_x86(): not implemented yet"))
}

// TODO: this function might be omitted in the future if this is merged to pelite internally
fn generate_guid(signature: GUID, age: u32) -> Result<String> {
    let uuid = Uuid::from_fields(
        signature.Data1,
        signature.Data2,
        signature.Data3,
        &signature.Data4,
    )?;

    Ok(format!(
        "{}{:X}",
        uuid.to_simple().to_string().to_uppercase(),
        age
    ))
}

#[derive(Debug, Clone)]
pub struct Win32GUID {
    pub file_name: String,
    pub guid: String,
}

pub fn find_guid<T: AccessVirtualMemory>(
    mem: &mut T,
    start_block: &StartBlock,
    kernel_base: Address,
    kernel_size: Length,
) -> Result<Win32GUID> {
    let mut pe_buf = vec![0; kernel_size.as_usize()];
    mem.virt_read_raw_into(start_block.arch, start_block.dtb, kernel_base, &mut pe_buf)?;

    let pe = PeView::from_bytes(&pe_buf)?;

    let debug = match pe.debug() {
        Ok(d) => d,
        Err(_) => return Err(Error::new("unable to read debug_data in pe header")),
    };

    let code_view = debug
        .iter()
        .map(|e| e.entry())
        .filter_map(std::result::Result::ok)
        .filter(|&e| e.as_code_view().is_some())
        .nth(0)
        .ok_or_else(|| Error::new("unable to find codeview debug_data entry"))?
        .as_code_view()
        .unwrap();

    let signature = match code_view {
        CodeView::Cv70 { image, .. } => image.Signature,
        CodeView::Cv20 { image: _, .. } => {
            return Err(Error::new(
                "invalid code_view entry version 2 found, expected 7",
            ))
        }
    };

    Ok(Win32GUID {
        file_name: code_view.pdb_file_name().to_string(),
        guid: generate_guid(signature, code_view.age())?,
    })
}
