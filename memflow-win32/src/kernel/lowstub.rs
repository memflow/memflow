mod x64;
mod x86;
mod x86pae;

use std::prelude::v1::*;

use crate::error::{Error, Result};

use log::warn;

use memflow_core::architecture::Architecture;
use memflow_core::mem::PhysicalMemory;
use memflow_core::types::{size, Address, PhysicalAddress};

// PROCESSOR_START_BLOCK
#[derive(Debug, Copy, Clone)]
pub struct StartBlock {
    pub arch: Architecture,
    pub kernel_hint: Address,
    pub dtb: Address,
}

// bcdedit /set firstmegabytepolicyuseall
pub fn find<T: PhysicalMemory + ?Sized>(
    mem: &mut T,
    arch: Option<Architecture>,
) -> Result<StartBlock> {
    if let Some(arch) = arch {
        match arch {
            Architecture::X64 => {
                // read low 1mb stub
                let mut low1m = vec![0; size::mb(1)];
                mem.phys_read_raw_into(PhysicalAddress::NULL, &mut low1m)?;

                // find x64 dtb in low stub < 1M
                match x64::find_lowstub(&low1m) {
                    Ok(d) => return Ok(d),
                    Err(e) => warn!("x64::find_lowstub() error: {}", e),
                }

                // read low 16mb stub
                let mut low16m = vec![0; size::mb(16)];
                mem.phys_read_raw_into(PhysicalAddress::NULL, &mut low16m)?;

                x64::find(&low16m)
            }
            Architecture::X86Pae => {
                let mut low16m = vec![0; size::mb(16)];
                mem.phys_read_raw_into(PhysicalAddress::NULL, &mut low16m)?;
                x86pae::find(&low16m)
            }
            Architecture::X86 => {
                let mut low16m = vec![0; size::mb(16)];
                mem.phys_read_raw_into(PhysicalAddress::NULL, &mut low16m)?;
                x86::find(&low16m)
            }
            _ => Err(Error::InvalidArchitecture),
        }
    } else {
        // check all architectures
        // read low 1mb stub
        let mut low1m = vec![0; size::mb(1)];
        mem.phys_read_raw_into(PhysicalAddress::NULL, &mut low1m)?;

        // find x64 dtb in low stub < 1M
        match x64::find_lowstub(&low1m) {
            Ok(d) => return Ok(d),
            Err(e) => warn!("x64::find_lowstub() error: {}", e),
        }

        // TODO: append instead of read twice?
        // read low 16mb stub
        let mut low16m = vec![0; size::mb(16)];
        mem.phys_read_raw_into(PhysicalAddress::NULL, &mut low16m)?;

        match x64::find(&low16m) {
            Ok(d) => return Ok(d),
            Err(e) => warn!("x64::find() error: {}", e),
        }

        match x86pae::find(&low16m) {
            Ok(d) => return Ok(d),
            Err(e) => warn!("x86pae::find() error: {}", e),
        }

        match x86::find(&low16m) {
            Ok(d) => return Ok(d),
            Err(e) => warn!("x86::find() error: {}", e),
        }

        Err(Error::Initialization("unable to find dtb"))
    }
}
