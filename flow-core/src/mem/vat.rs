use crate::error::{Error, Result};

use log::trace;

use crate::address::{Address, Length};
use crate::arch::Architecture;
use crate::mem::AccessPhysicalMemory;

#[allow(unused)]
pub fn virt_read_raw_into<T: AccessPhysicalMemory>(
    mem: &mut T,
    arch: Architecture,
    dtb: Address,
    addr: Address,
    out: &mut [u8],
) -> Result<()> {
    let mut base = addr;
    let end = base + Length::from(out.len());

    // pre-allocate buffer
    let page_size = arch.page_size();
    let mut buf = vec![0u8; page_size.as_usize()];

    while base < end {
        let mut aligned_len = (base + page_size).as_page_aligned(page_size) - base;
        if base + aligned_len > end {
            aligned_len = end - base;
        }

        let pa = arch.vtop(mem, dtb, base);
        if let Ok(pa) = pa {
            mem.phys_read_raw_into(pa, &mut buf[..aligned_len.as_usize()])?;
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

#[allow(unused)]
pub fn virt_write_raw<T: AccessPhysicalMemory>(
    mem: &mut T,
    arch: Architecture,
    dtb: Address,
    addr: Address,
    data: &[u8],
) -> Result<()> {
    let pa = arch.vtop(mem, dtb, addr)?;
    if pa.is_null() {
        // TODO: add more debug info
        Err(Error::new(
            "virt_write(): unable to resolve physical address",
        ))
    } else {
        mem.phys_write_raw(pa, data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::address::Address;
    use crate::arch::Architecture;
    use crate::mem::AccessVirtualMemory;

    impl AccessPhysicalMemory for Vec<u8> {
        fn phys_read_raw_into(&mut self, addr: Address, out: &mut [u8]) -> Result<()> {
            out.copy_from_slice(&self[addr.as_usize()..(addr.as_usize() + out.len())]);
            Ok(())
        }

        fn phys_write_raw(&mut self, _addr: Address, _data: &[u8]) -> Result<()> {
            Err(Error::new("phys_write not implemented"))
        }
    }

    impl AccessVirtualMemory for Vec<u8> {
        fn virt_read_raw_into(
            &mut self,
            arch: Architecture,
            dtb: Address,
            addr: Address,
            out: &mut [u8],
        ) -> Result<()> {
            virt_read_raw_into(self, arch, dtb, addr, out)
        }

        fn virt_write_raw(
            &mut self,
            arch: Architecture,
            dtb: Address,
            addr: Address,
            data: &[u8],
        ) -> Result<()> {
            virt_write_raw(self, arch, dtb, addr, data)
        }
    }

    #[test]
    fn test_virt_read_small() {
        let mut buf = vec![0u8; 256];
        for i in 0..buf.len() {
            buf[i] = i as u8;
        }

        let mut out = vec![0u8; buf.len()];
        buf.virt_read_into(
            Architecture::Null,
            Address::from(0),
            Address::from(0),
            &mut out[..],
        )
        .unwrap();
        assert_eq!(buf, out);
    }

    #[test]
    fn test_virt_read_medium() {
        let mut buf = vec![0u8; 0x1000];
        for i in 0..buf.len() {
            buf[i] = i as u8;
        }

        let mut out = vec![0u8; buf.len()];
        buf.virt_read_into(
            Architecture::Null,
            Address::from(0),
            Address::from(0),
            &mut out[..],
        )
        .unwrap();
        assert_eq!(buf, out);
    }

    #[test]
    fn test_virt_read_big() {
        let mut buf = vec![0u8; 16 * 0x1000];
        for i in 0..buf.len() {
            buf[i] = i as u8;
        }

        let mut out = vec![0u8; buf.len()];
        buf.virt_read_into(
            Architecture::Null,
            Address::from(0),
            Address::from(0),
            &mut out[..],
        )
        .unwrap();
        assert_eq!(buf, out);
    }
}
