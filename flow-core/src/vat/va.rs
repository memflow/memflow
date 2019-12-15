use crate::error::{Error, Result};

use crate::address::{Address, Length};
use crate::arch::Architecture;
use crate::mem::{PhysicalRead, PhysicalWrite, VirtualRead, VirtualWrite};

use crate::vat::VirtualAddressTranslation;

// TODO: find a cleaner way to do this?
pub struct VatImpl<'a, T: PhysicalRead + VirtualAddressTranslation>(pub &'a mut T);

impl<'a, T: PhysicalRead + VirtualAddressTranslation> VatImpl<'a, T> {
    pub fn new(mem: &'a mut T) -> Self {
        VatImpl { 0: mem }
    }
}

// TODO: recover from vtop failures if we request to much memory!
impl<'a, T: PhysicalRead + VirtualAddressTranslation> VirtualRead for VatImpl<'a, T> {
    fn virt_read(
        &mut self,
        arch: Architecture,
        dtb: Address,
        addr: Address,
        len: Length,
    ) -> Result<Vec<u8>> {
        let mut result: Vec<u8> = vec![0; len.as_usize()];

        let mut base = addr;
        let end = addr + len;
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
            if !pa.is_null() {
                let mem = self.0.phys_read(pa, aligned_len)?;
                let start = (base - addr).as_usize();
                mem.iter().enumerate().for_each(|(i, b)| {
                    result[start + i] = *b;
                });
            } else {
                // skip
            }

            base += aligned_len;
        }

        Ok(result)
    }
}

impl<'a, T: PhysicalRead + PhysicalWrite + VirtualAddressTranslation> VirtualWrite
    for VatImpl<'a, T>
{
    fn virt_write(
        &mut self,
        arch: Architecture,
        dtb: Address,
        addr: Address,
        data: &[u8],
    ) -> Result<Length> {
        let pa = self.0.vtop(arch, dtb, addr)?;
        if !pa.is_null() {
            self.0.phys_write(pa, data)
        } else {
            // TODO: add more debug info
            Err(Error::new(
                "virt_write(): unable to resolve physical address",
            ))
        }
    }
}
