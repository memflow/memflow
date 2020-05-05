use super::*;
use crate::address::{Address, PhysicalAddress};
use crate::arch::Architecture;
use crate::mem::AccessVirtualMemory;

impl AccessPhysicalMemory for Vec<u8> {
    fn phys_read_raw_into(&mut self, addr: PhysicalAddress, out: &mut [u8]) -> Result<()> {
        out.copy_from_slice(&self[addr.as_usize()..(addr.as_usize() + out.len())]);
        Ok(())
    }

    fn phys_write_raw(&mut self, addr: PhysicalAddress, data: &[u8]) -> Result<()> {
        self[addr.as_usize()..(addr.as_usize() + data.len())].copy_from_slice(&data);
        Ok(())
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

    fn virt_page_info(&mut self, arch: Architecture, dtb: Address, addr: Address) -> Result<Page> {
        virt_page_info(self, arch, dtb, addr)
    }
}

#[test]
fn test_virt_read_small() {
    let mut buf = vec![0u8; 256];
    for (i, item) in buf.iter_mut().enumerate() {
        *item = i as u8;
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
fn test_virt_write_small() {
    let mut buf = vec![0u8; 256];
    let mut input = vec![0u8; buf.len()];
    for (i, item) in input.iter_mut().enumerate() {
        *item = i as u8;
    }
    buf.virt_write(
        Architecture::Null,
        Address::from(0),
        Address::from(0),
        &input[..],
    )
    .unwrap();
    assert_eq!(buf.len(), input.len());
    assert_eq!(buf, input);
}

#[test]
fn test_virt_read_small_shifted() {
    let mut buf = vec![0u8; 256];
    for (i, item) in buf.iter_mut().enumerate() {
        *item = i as u8;
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
fn test_virt_write_small_shifted() {
    let mut buf = vec![0u8; 256];
    let mut input = vec![0u8; buf.len() - 128];
    for (i, item) in input.iter_mut().enumerate() {
        *item = i as u8;
    }
    buf.virt_write(
        Architecture::Null,
        Address::from(0),
        Address::from(128),
        &input[..],
    )
    .unwrap();
    assert_eq!(buf[128..].to_vec().len(), input.len());
    assert_eq!(buf[128..].to_vec(), input);
}

#[test]
fn test_virt_read_medium() {
    let mut buf = vec![0u8; 0x1000];
    for (i, item) in buf.iter_mut().enumerate() {
        *item = i as u8;
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
fn test_virt_write_medium() {
    let mut buf = vec![0u8; 0x1000];
    let mut input = vec![0u8; buf.len()];
    for (i, item) in input.iter_mut().enumerate() {
        *item = i as u8;
    }
    buf.virt_write(
        Architecture::Null,
        Address::from(0),
        Address::from(0),
        &input[..],
    )
    .unwrap();
    assert_eq!(buf.len(), input.len());
    assert_eq!(buf, input);
}

#[test]
fn test_virt_read_medium_shifted() {
    let mut buf = vec![0u8; 0x1000];
    for (i, item) in buf.iter_mut().enumerate() {
        *item = i as u8;
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
fn test_virt_write_medium_shifted() {
    let mut buf = vec![0u8; 0x1000];
    let mut input = vec![0u8; buf.len() - 0x100];
    for (i, item) in input.iter_mut().enumerate() {
        *item = i as u8;
    }
    buf.virt_write(
        Architecture::Null,
        Address::from(0),
        Address::from(0x100),
        &input[..],
    )
    .unwrap();
    assert_eq!(buf[0x100..].to_vec().len(), input.len());
    assert_eq!(buf[0x100..].to_vec(), input);
}

#[test]
fn test_virt_read_big() {
    let mut buf = vec![0u8; 16 * 0x1000];
    for (i, item) in buf.iter_mut().enumerate() {
        *item = i as u8;
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
fn test_virt_write_big() {
    let mut buf = vec![0u8; 16 * 0x1000];
    let mut input = vec![0u8; buf.len()];
    for (i, item) in input.iter_mut().enumerate() {
        *item = i as u8;
    }
    buf.virt_write(
        Architecture::Null,
        Address::from(0),
        Address::from(0),
        &input[..],
    )
    .unwrap();
    assert_eq!(buf.len(), input.len());
    assert_eq!(buf, input);
}

#[test]
fn test_virt_read_big_shifted() {
    let mut buf = vec![0u8; 16 * 0x1000];
    for (i, item) in buf.iter_mut().enumerate() {
        *item = i as u8;
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
fn test_virt_write_big_shifted() {
    let mut buf = vec![0u8; 16 * 0x1000];
    let mut input = vec![0u8; buf.len() - 0x100];
    for (i, item) in input.iter_mut().enumerate() {
        *item = i as u8;
    }
    buf.virt_write(
        Architecture::Null,
        Address::from(0),
        Address::from(0x100),
        &input[..],
    )
    .unwrap();
    assert_eq!(buf[0x100..].to_vec().len(), input.len());
    assert_eq!(buf[0x100..].to_vec(), input);
}
