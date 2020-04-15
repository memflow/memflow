use crate::error::{Error, Result};

use log::trace;

use crate::address::{Address, Length};
use crate::arch::Architecture;
use crate::mem::*;

use crate::vat::VirtualAddressTranslation;

// TODO: find a cleaner way to do this?
pub struct VatImpl<'a, T: AccessPhysicalMemory + VirtualAddressTranslation>(pub &'a mut T);

impl<'a, T: AccessPhysicalMemory + VirtualAddressTranslation> VatImpl<'a, T> {
    pub fn new(mem: &'a mut T) -> Self {
        VatImpl { 0: mem }
    }
}

// TODO: recover from vtop failures if we request to much memory!
impl<'a, T: AccessPhysicalMemory + VirtualAddressTranslation> AccessVirtualMemory
    for VatImpl<'a, T>
{
    fn virt_read_raw_into(
        &mut self,
        arch: Architecture,
        dtb: Address,
        addr: Address,
        out: &mut [u8],
    ) -> Result<()> {
        let mut base = addr;
        let end = addr + Length::from(out.len());

        // pre-allocate buffer
        let mut buf = vec![0; Length::from_kb(4).as_usize()];

        while base < end {
            let mut aligned_len = (base + Length::from_kb(4))
                .as_page_aligned(arch.instruction_set.page_size())
                - base;
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
                self.0
                    .phys_read_raw_into(pa, &mut buf[..aligned_len.as_usize()])?;
                let offset = (base - addr).as_usize();
                unsafe {
                    std::ptr::copy_nonoverlapping(
                        buf.as_ptr(),
                        out[offset..].as_mut_ptr(),
                        aligned_len.as_usize(),
                    );
                }
            }

            base += aligned_len;
        }

        Ok(())
    }

    // TODO: see above
    fn virt_write_raw(
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
            self.0.phys_write_raw(pa, data)
        }
    }
}
