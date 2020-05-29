mod x64;
mod x86;
mod x86pae;

use crate::error::{Error, Result};

use log::warn;

use flow_core::architecture::Architecture;
use flow_core::mem::AccessPhysicalMemoryExt;
use flow_core::types::{Address, Length, PhysicalAddress};

// PROCESSOR_START_BLOCK
#[derive(Debug, Copy, Clone)]
pub struct StartBlock {
    pub arch: Architecture,
    pub va: Address,
    pub dtb: Address,
}

// bcdedit /set firstmegabytepolicyuseall
pub fn find<T: AccessPhysicalMemoryExt + ?Sized>(mem: &mut T) -> Result<StartBlock> {
    // read low 1mb stub
    let mut low1m = vec![0; Length::from_mb(1).as_usize()];
    mem.phys_read_raw_into(PhysicalAddress::NULL, &mut low1m)?;

    // find x64 dtb in low stub < 1M
    match x64::find_lowstub(&low1m) {
        Ok(d) => return Ok(d),
        Err(e) => warn!("x64::find_lowstub() error: {}", e),
    }

    // TODO: append instead of read twice?
    // read low 16mb stub
    let mut low16m = vec![0; Length::from_mb(16).as_usize()];
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

    Err(Error::new("unable to find dtb"))
}
