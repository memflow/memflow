use crate::error::{Error, Result};

use log::trace;

use crate::address::{Address, Length};
use crate::arch::Architecture;
use crate::mem::*;

// TODO: find a cleaner way to do this?
pub struct VatImpl<'a, T: AccessPhysicalMemory>(pub &'a mut T);

impl<'a, T: AccessPhysicalMemory> VatImpl<'a, T> {
    pub fn new(mem: &'a mut T) -> Self {
        VatImpl { 0: mem }
    }
}

// TODO: recover from vtop failures if we request to much memory!
impl<'a, T: AccessPhysicalMemory> AccessVirtualMemory for VatImpl<'a, T> {
    fn virt_read_raw_into(
        &mut self,
        arch: Architecture,
        dtb: Address,
        addr: Address,
        out: &mut [u8],
    ) -> Result<()> {
        let mut base = addr;
        let end = base + Length::from(out.len());

        // pre-allocate buffer
        let page_size = arch.instruction_set.page_size();
        let mut buf = vec![0u8; page_size.as_usize()];

        while base < end {
            let mut aligned_len = (base + page_size).as_page_aligned(page_size) - base;
            if base + aligned_len > end {
                aligned_len = end - base;
            }

            let pa = arch.instruction_set.vtop(self.0, dtb, base);
            if let Ok(pa) = pa {
                self.0
                    .phys_read_raw_into(pa, &mut buf[..aligned_len.as_usize()])?;
                let offset = (base - addr).as_usize();
                out[offset..(offset + aligned_len.as_usize())]
                    .copy_from_slice(&buf[..aligned_len.as_usize()]);
            } else {
                // skip
                trace!("pa is null, skipping page");
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
        let pa = arch.instruction_set.vtop(self.0, dtb, addr)?;
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

#[cfg(test)]
mod tests {
    use super::{AccessPhysicalMemory, AccessVirtualMemory, VatImpl};
    use crate::address::Address;
    use crate::arch::{Architecture, InstructionSet};
    use crate::error::*;

    pub struct VirtualMemoryMock<'a> {
        pub buf: &'a [u8],
    }

    impl<'a> AccessPhysicalMemory for VirtualMemoryMock<'a> {
        fn phys_read_raw_into(&mut self, addr: Address, out: &mut [u8]) -> Result<()> {
            out.copy_from_slice(&self.buf[addr.as_usize()..(addr.as_usize() + out.len())]);
            Ok(())
        }

        fn phys_write_raw(&mut self, _addr: Address, _data: &[u8]) -> Result<()> {
            Err(Error::new("phys_write not implemented"))
        }
    }

    impl<'a> AccessVirtualMemory for VirtualMemoryMock<'a> {
        fn virt_read_raw_into(
            &mut self,
            arch: Architecture,
            dtb: Address,
            addr: Address,
            out: &mut [u8],
        ) -> Result<()> {
            VatImpl::new(self).virt_read_raw_into(arch, dtb, addr, out)
        }

        fn virt_write_raw(
            &mut self,
            arch: Architecture,
            dtb: Address,
            addr: Address,
            data: &[u8],
        ) -> Result<()> {
            VatImpl::new(self).virt_write_raw(arch, dtb, addr, data)
        }
    }

    #[test]
    fn test_virt_read() {
        // TODO
        let mut buf = vec![0u8; 16 * 0x1000];
        for i in 0..buf.len() {
            buf[i] = i as u8;
        }

        let mut va = VirtualMemoryMock { buf: &buf };

        let mut out = vec![0u8; buf.len()];
        va.virt_read_into(
            Architecture::from(InstructionSet::Null),
            Address::from(0),
            Address::from(0),
            &mut out[..],
        )
        .unwrap();
        assert_eq!(buf, out);
    }
}
