use crate::error::{Error, Result};

use log::trace;

use crate::address::{Address, Length};
use crate::arch::Architecture;
use crate::mem::*;

use crate::vat::VirtualAddressTranslation;

// TODO: find a cleaner way to do this?
pub struct VatImpl<'a, T: PhysicalMemoryTrait + VirtualAddressTranslation>(pub &'a mut T);

impl<'a, T: PhysicalMemoryTrait + VirtualAddressTranslation> VatImpl<'a, T> {
    pub fn new(mem: &'a mut T) -> Self {
        VatImpl { 0: mem }
    }
}

// TODO: recover from vtop failures if we request to much memory!
impl<'a, T: PhysicalMemoryTrait + VirtualAddressTranslation> VirtualMemoryTrait for VatImpl<'a, T> {
    fn virt_read(
        &mut self,
        arch: Architecture,
        dtb: Address,
        addr: Address,
        out: &mut [u8],
    ) -> Result<()> {
        let mut base = addr;
        let end = addr + Length::from(out.len());
        while base < end {
            let mut aligned_len = (addr + Length::from_kb(4))
                .as_page_aligned(arch.instruction_set.page_size())
                - addr;
            if base + aligned_len > end {
                aligned_len = end - base;
            }

            let pa = self
                .0
                .vtop(arch, dtb, base)
                .unwrap_or_else(|_| Address::null());
            if pa.is_null() {
                // skip
                trace!("pa is null, skipping page");
            } else {
                let mut buf = vec![0; aligned_len.as_usize()];
                self.0.phys_read(pa, &mut buf)?;
                let start = (base - addr).as_usize();
                buf.iter().enumerate().for_each(|(i, b)| {
                    out[start + i] = *b;
                });
            }

            base += aligned_len;
        }

        Ok(())
    }

    fn virt_write(
        &mut self,
        arch: Architecture,
        dtb: Address,
        addr: Address,
        data: &[u8],
    ) -> Result<()> {
        let pa = self.0.vtop(arch, dtb, addr)?;
        if pa.is_null() {
            // TODO: add more debug info
            Err(Error::new(
                "virt_write(): unable to resolve physical address",
            ))
        } else {
            self.0.phys_write(pa, data)
        }
    }
}
