mod x64;
mod x86;
mod x86pae;

use std::prelude::v1::*;

use crate::error::{Error, Result};

use log::warn;

use memflow::architecture;
use memflow::architecture::ArchitectureObj;
use memflow::mem::PhysicalMemory;
use memflow::types::{size, Address, PhysicalAddress};

// PROCESSOR_START_BLOCK
#[derive(Debug, Copy, Clone)]
#[cfg_attr(feature = "serde", derive(::serde::Serialize))]
pub struct StartBlock {
    pub arch: ArchitectureObj,
    pub kernel_hint: Address,
    pub dtb: Address,
}

pub fn find_fallback<T: PhysicalMemory>(mem: &mut T, arch: ArchitectureObj) -> Result<StartBlock> {
    if arch == architecture::x86::x64::ARCH {
        // read low 16mb stub
        let mut low16m = vec![0; size::mb(16)];
        mem.phys_read_raw_into(PhysicalAddress::NULL, &mut low16m)?;

        x64::find(&low16m)
    } else {
        Err(Error::Initialization(
            "start_block: fallback not implemented for given arch",
        ))
    }
}

// bcdedit /set firstmegabytepolicyuseall
pub fn find<T: PhysicalMemory>(mem: &mut T, arch: Option<ArchitectureObj>) -> Result<StartBlock> {
    if let Some(arch) = arch {
        if arch == architecture::x86::x64::ARCH {
            // read low 1mb stub
            let mut low1m = vec![0; size::mb(1)];
            mem.phys_read_raw_into(PhysicalAddress::NULL, &mut low1m)?;

            // find x64 dtb in low stub < 1M
            match x64::find_lowstub(&low1m) {
                Ok(d) => {
                    if d.dtb.as_u64() != 0 {
                        return Ok(d);
                    }
                }
                Err(e) => warn!("x64::find_lowstub() error: {}", e),
            }

            find_fallback(mem, arch)
        } else if arch == architecture::x86::x32_pae::ARCH {
            let mut low16m = vec![0; size::mb(16)];
            mem.phys_read_raw_into(PhysicalAddress::NULL, &mut low16m)?;
            x86pae::find(&low16m)
        } else if arch == architecture::x86::x32::ARCH {
            let mut low16m = vec![0; size::mb(16)];
            mem.phys_read_raw_into(PhysicalAddress::NULL, &mut low16m)?;
            x86::find(&low16m)
        } else {
            Err(Error::InvalidArchitecture)
        }
    } else {
        find(mem, Some(architecture::x86::x64::ARCH))
            .or_else(|_| find(mem, Some(architecture::x86::x32_pae::ARCH)))
            .or_else(|_| find(mem, Some(architecture::x86::x32::ARCH)))
            .map_err(|_| Error::Initialization("unable to find dtb"))
    }
}
