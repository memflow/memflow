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
    assert_eq!(buf.len(), out.len());
    assert_eq!(buf, out);
}

#[test]
fn test_virt_read_small_shifted() {
    let mut buf = vec![0u8; 256];
    for i in 0..buf.len() {
        buf[i] = i as u8;
    }

    let mut out = vec![0u8; buf.len() - 128];
    buf.virt_read_into(
        Architecture::Null,
        Address::from(0),
        Address::from(128),
        &mut out[..],
    )
    .unwrap();
    assert_eq!(buf[128..].to_vec().len(), out.len());
    assert_eq!(buf[128..].to_vec(), out);
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
    assert_eq!(buf.len(), out.len());
    assert_eq!(buf, out);
}

#[test]
fn test_virt_read_medium_shifted() {
    let mut buf = vec![0u8; 0x1000];
    for i in 0..buf.len() {
        buf[i] = i as u8;
    }

    let mut out = vec![0u8; buf.len() - 0x100];
    buf.virt_read_into(
        Architecture::Null,
        Address::from(0),
        Address::from(0x100),
        &mut out[..],
    )
    .unwrap();
    assert_eq!(buf[0x100..].to_vec().len(), out.len());
    assert_eq!(buf[0x100..].to_vec(), out);
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
    assert_eq!(buf.len(), out.len());
    assert_eq!(buf, out);
}

#[test]
fn test_virt_read_big_shifted() {
    let mut buf = vec![0u8; 16 * 0x1000];
    for i in 0..buf.len() {
        buf[i] = i as u8;
    }

    let mut out = vec![0u8; buf.len() - 0x100];
    buf.virt_read_into(
        Architecture::Null,
        Address::from(0),
        Address::from(0x100),
        &mut out[..],
    )
    .unwrap();
    assert_eq!(buf[0x100..].to_vec().len(), out.len());
    assert_eq!(buf[0x100..].to_vec(), out);
}
