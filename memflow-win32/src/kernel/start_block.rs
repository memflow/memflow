mod aarch64;
mod x64;
mod x86;
mod x86pae;

use std::prelude::v1::*;

use log::warn;

use memflow::architecture::ArchitectureIdent;
use memflow::error::{Error, ErrorKind, ErrorOrigin, Result};
use memflow::mem::PhysicalMemory;
use memflow::types::{size, Address, PhysicalAddress};

// PROCESSOR_START_BLOCK
#[derive(Debug, Copy, Clone)]
#[cfg_attr(feature = "serde", derive(::serde::Serialize))]
pub struct StartBlock {
    pub arch: ArchitectureIdent,
    pub kernel_hint: Address,
    pub dtb: Address,
}

pub fn find_fallback<T: PhysicalMemory>(
    mem: &mut T,
    arch: ArchitectureIdent,
) -> Result<StartBlock> {
    match arch {
        ArchitectureIdent::X86(64, _) => {
            // read low 16mb stub
            let mut low16m = vec![0; size::mb(16)];
            mem.phys_read_into(PhysicalAddress::NULL, low16m.as_mut_slice())?;

            x64::find(&low16m)
        }
        ArchitectureIdent::AArch64(_) => {
            // read low 16mb stub
            let mut low16m = vec![0; size::mb(16)];

            //TODO: configure this, but so far arm null starts at this address
            mem.phys_read_into(aarch64::PHYS_BASE.into(), low16m.as_mut_slice())?;

            aarch64::find(&low16m)
        }
        _ => Err(Error(ErrorOrigin::OsLayer, ErrorKind::NotImplemented)
            .log_error("start_block: fallback not implemented for given arch")),
    }
}

// bcdedit /set firstmegabytepolicyuseall
pub fn find<T: PhysicalMemory>(mem: &mut T, arch: Option<ArchitectureIdent>) -> Result<StartBlock> {
    if let Some(arch) = arch {
        match arch {
            ArchitectureIdent::X86(64, _) => {
                // read low 1mb stub
                let mut low1m = vec![0; size::mb(1)];
                mem.phys_read_into(PhysicalAddress::NULL, low1m.as_mut_slice())?;

                // find x64 dtb in low stub < 1M
                match x64::find_lowstub(&low1m) {
                    Ok(d) => {
                        if d.dtb.to_umem() != 0 {
                            return Ok(d);
                        }
                    }
                    Err(e) => warn!("x64::find_lowstub() error: {}", e),
                }

                find_fallback(mem, arch)
            }
            ArchitectureIdent::X86(32, true) => {
                let mut low16m = vec![0; size::mb(16)];
                mem.phys_read_into(PhysicalAddress::NULL, low16m.as_mut_slice())?;
                x86pae::find(&low16m)
            }
            ArchitectureIdent::X86(32, false) => {
                let mut low16m = vec![0; size::mb(16)];
                mem.phys_read_into(PhysicalAddress::NULL, low16m.as_mut_slice())?;
                x86::find(&low16m)
            }
            ArchitectureIdent::AArch64(_) => find_fallback(mem, arch),
            _ => Err(Error(ErrorOrigin::OsLayer, ErrorKind::NotSupported)
                .log_error("Unsupported architecture")),
        }
    } else {
        find(mem, Some(ArchitectureIdent::X86(64, false)))
            .or_else(|_| find(mem, Some(ArchitectureIdent::X86(32, true))))
            .or_else(|_| find(mem, Some(ArchitectureIdent::X86(32, false))))
            .or_else(|_| find(mem, Some(ArchitectureIdent::AArch64(size::kb(4)))))
            .map_err(|_| {
                Error(ErrorOrigin::OsLayer, ErrorKind::NotFound).log_error("unable to find dtb")
            })
    }
}
