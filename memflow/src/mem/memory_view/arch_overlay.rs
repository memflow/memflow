//! Overlays a custom architecture on the memory view

use super::*;
use crate::architecture::{ArchitectureObj, Endianess};
use crate::error::*;

/// Allows to overwrite the architecture of the memory view.
///
/// Is useful when a 32 bit process runs in a 64 bit architecture, and a 64-bit Pointer is wanted
/// to be read with `read_ptr`.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct ArchOverlayView<T> {
    mem: T,
    arch_bits: u8,
    little_endian: bool,
}

impl<T: MemoryView> ArchOverlayView<T> {
    pub fn new_parts(mem: T, arch_bits: u8, little_endian: bool) -> Self {
        Self {
            mem,
            arch_bits,
            little_endian,
        }
    }

    pub fn new(mem: T, arch: ArchitectureObj) -> Self {
        Self::new_parts(
            mem,
            arch.bits(),
            arch.endianess() == Endianess::LittleEndian,
        )
    }
}

impl<T: MemoryView> MemoryView for ArchOverlayView<T> {
    fn read_raw_iter(&mut self, data: ReadRawMemOps) -> Result<()> {
        self.mem.read_raw_iter(data)
    }

    fn write_raw_iter(&mut self, data: WriteRawMemOps) -> Result<()> {
        self.mem.write_raw_iter(data)
    }

    fn metadata(&self) -> MemoryViewMetadata {
        MemoryViewMetadata {
            little_endian: self.little_endian,
            arch_bits: self.arch_bits,
            ..self.mem.metadata()
        }
    }
}
