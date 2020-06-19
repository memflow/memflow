use crate::architecture::Architecture;
use crate::dummy::DummyMemory;
use crate::mem::{TranslateArch, VirtualFromPhysical, VirtualMemory, VirtualTranslate};
use crate::types::size;

#[test]
fn test_vtop() {
    let mut dummy_mem = DummyMemory::new(size::mb(512));
    let virt_size = size::mb(8);
    let (dtb, virt_base) = dummy_mem.alloc_dtb(virt_size, &[]);
    let arch = Architecture::X64;
    let mut vat = TranslateArch::new(arch);

    for i in (0..virt_size).step_by(128) {
        let virt_base = virt_base + i;
        let vtop = match vat.virt_to_phys(&mut dummy_mem, dtb, virt_base) {
            Err(_) => None,
            Ok(paddr) => Some(paddr.address()),
        };
        let dummy_vtop = dummy_mem.vtop(dtb, virt_base);

        assert_eq!(vtop, dummy_vtop);
    }

    for i in 0..128 {
        let virt_base = virt_base + virt_size + i;
        let vtop = match vat.virt_to_phys(&mut dummy_mem, dtb, virt_base) {
            Err(_) => None,
            Ok(paddr) => Some(paddr.address()),
        };
        let dummy_vtop = dummy_mem.vtop(dtb, virt_base);

        assert!(vtop.is_none());

        assert_eq!(vtop, dummy_vtop);
    }

    for i in 0..128 {
        let virt_base = virt_base - i;
        let vtop = match vat.virt_to_phys(&mut dummy_mem, dtb, virt_base) {
            Err(_) => None,
            Ok(paddr) => Some(paddr.address()),
        };
        let dummy_vtop = dummy_mem.vtop(dtb, virt_base);

        assert!(i == 0 || vtop.is_none());

        assert_eq!(vtop, dummy_vtop);
    }
}

#[test]
fn test_virt_read_small() {
    let mut dummy_mem = DummyMemory::new(size::mb(2));
    let mut buf = vec![0u8; 256];
    for (i, item) in buf.iter_mut().enumerate() {
        *item = i as u8;
    }
    let (dtb, virt_base) = dummy_mem.alloc_dtb(buf.len().into(), &buf);
    let arch = Architecture::X64;
    let mut virt_mem = VirtualFromPhysical::new(&mut dummy_mem, arch, arch, dtb);

    let mut out = vec![0u8; buf.len()];
    virt_mem.virt_read_into(virt_base, &mut out[..]).unwrap();
    assert_eq!(buf.len(), out.len());
    assert_eq!(buf, out);
}

#[test]
fn test_virt_write_small() {
    let mut dummy_mem = DummyMemory::new(size::mb(2));
    let mut buf = vec![0u8; 256];
    let mut input = vec![0u8; buf.len()];
    for (i, item) in input.iter_mut().enumerate() {
        *item = i as u8;
    }
    let (dtb, virt_base) = dummy_mem.alloc_dtb(input.len().into(), &input);
    let arch = Architecture::X64;
    let mut virt_mem = VirtualFromPhysical::new(&mut dummy_mem, arch, arch, dtb);

    virt_mem.virt_write(virt_base, &input[..]).unwrap();
    virt_mem.virt_read_into(virt_base, &mut buf[..]).unwrap();
    assert_eq!(buf.len(), input.len());
    assert_eq!(buf, input);
}

#[test]
fn test_virt_read_small_shifted() {
    let mut dummy_mem = DummyMemory::new(size::mb(2));
    let mut buf = vec![0u8; 256];
    for (i, item) in buf.iter_mut().enumerate() {
        *item = i as u8;
    }
    let (dtb, virt_base) = dummy_mem.alloc_dtb(buf.len().into(), &buf);
    let arch = Architecture::X64;
    let mut virt_mem = VirtualFromPhysical::new(&mut dummy_mem, arch, arch, dtb);

    let mut out = vec![0u8; buf.len() - 128];
    virt_mem
        .virt_read_into(virt_base + 128, &mut out[..])
        .unwrap();
    assert_eq!(buf[128..].to_vec().len(), out.len());
    assert_eq!(buf[128..].to_vec(), out);
}

#[test]
fn test_virt_write_small_shifted() {
    let mut dummy_mem = DummyMemory::new(size::mb(2));
    let mut buf = vec![0u8; 128];
    let mut input = vec![0u8; buf.len()];
    for (i, item) in input.iter_mut().enumerate() {
        *item = i as u8;
    }
    let (dtb, virt_base) = dummy_mem.alloc_dtb(input.len().into(), &input);
    let arch = Architecture::X64;
    let mut virt_mem = VirtualFromPhysical::new(&mut dummy_mem, arch, arch, dtb);

    virt_mem.virt_write(virt_base + 128, &input[..]).unwrap();
    virt_mem
        .virt_read_into(virt_base + 128, &mut buf[..])
        .unwrap();
    assert_eq!(buf.to_vec().len(), input.len());
    assert_eq!(buf.to_vec(), input);
}

#[test]
fn test_virt_read_medium() {
    let mut dummy_mem = DummyMemory::new(size::mb(2));
    let mut buf = vec![0u8; 0x1000];
    for (i, item) in buf.iter_mut().enumerate() {
        *item = i as u8;
    }
    let (dtb, virt_base) = dummy_mem.alloc_dtb(buf.len().into(), &buf);
    let arch = Architecture::X64;
    let mut virt_mem = VirtualFromPhysical::new(&mut dummy_mem, arch, arch, dtb);

    let mut out = vec![0u8; buf.len()];
    virt_mem.virt_read_into(virt_base, &mut out[..]).unwrap();
    assert_eq!(buf.len(), out.len());
    assert_eq!(buf, out);
}

#[test]
fn test_virt_write_medium() {
    let mut dummy_mem = DummyMemory::new(size::mb(2));
    let mut buf = vec![0u8; 0x1000];
    let mut input = vec![0u8; buf.len()];
    for (i, item) in input.iter_mut().enumerate() {
        *item = i as u8;
    }
    let (dtb, virt_base) = dummy_mem.alloc_dtb(input.len().into(), &input);
    let arch = Architecture::X64;
    let mut virt_mem = VirtualFromPhysical::new(&mut dummy_mem, arch, arch, dtb);

    virt_mem.virt_write(virt_base, &input[..]).unwrap();
    virt_mem.virt_read_into(virt_base, &mut buf[..]).unwrap();
    assert_eq!(buf.len(), input.len());
    assert_eq!(buf, input);
}

#[test]
fn test_virt_read_medium_shifted() {
    let mut dummy_mem = DummyMemory::new(size::mb(2));
    let mut buf = vec![0u8; 0x1000];
    for (i, item) in buf.iter_mut().enumerate() {
        *item = i as u8;
    }
    let (dtb, virt_base) = dummy_mem.alloc_dtb(buf.len().into(), &buf);
    let arch = Architecture::X64;
    let mut virt_mem = VirtualFromPhysical::new(&mut dummy_mem, arch, arch, dtb);

    let mut out = vec![0u8; buf.len() - 0x100];
    virt_mem
        .virt_read_into(virt_base + 0x100, &mut out[..])
        .unwrap();
    assert_eq!(buf[0x100..].to_vec().len(), out.len());
    assert_eq!(buf[0x100..].to_vec(), out);
}

#[test]
fn test_virt_write_medium_shifted() {
    let mut dummy_mem = DummyMemory::new(size::mb(2));
    let mut buf = vec![0u8; 0x1000 - 0x100];
    let mut input = vec![0u8; buf.len()];
    for (i, item) in input.iter_mut().enumerate() {
        *item = i as u8;
    }
    let (dtb, virt_base) = dummy_mem.alloc_dtb(input.len().into(), &input);
    let arch = Architecture::X64;
    let mut virt_mem = VirtualFromPhysical::new(&mut dummy_mem, arch, arch, dtb);

    virt_mem.virt_write(virt_base + 0x100, &input[..]).unwrap();
    virt_mem
        .virt_read_into(virt_base + 0x100, &mut buf[..])
        .unwrap();
    assert_eq!(buf.to_vec().len(), input.len());
    assert_eq!(buf.to_vec(), input);
}

#[test]
fn test_virt_read_big() {
    let mut dummy_mem = DummyMemory::new(size::mb(2));
    let mut buf = vec![0u8; 0x1000 * 16];
    for (i, item) in buf.iter_mut().enumerate() {
        *item = i as u8;
    }
    let (dtb, virt_base) = dummy_mem.alloc_dtb(buf.len().into(), &buf);
    let arch = Architecture::X64;
    let mut virt_mem = VirtualFromPhysical::new(&mut dummy_mem, arch, arch, dtb);

    let mut out = vec![0u8; buf.len()];
    virt_mem.virt_read_into(virt_base, &mut out[..]).unwrap();
    assert_eq!(buf.len(), out.len());
    assert_eq!(buf, out);
}

#[test]
fn test_virt_write_big() {
    let mut dummy_mem = DummyMemory::new(size::mb(2));
    let mut buf = vec![0u8; 0x1000 * 16];
    let mut input = vec![0u8; buf.len()];
    for (i, item) in input.iter_mut().enumerate() {
        *item = i as u8;
    }
    let (dtb, virt_base) = dummy_mem.alloc_dtb(input.len().into(), &input);
    let arch = Architecture::X64;
    let mut virt_mem = VirtualFromPhysical::new(&mut dummy_mem, arch, arch, dtb);

    virt_mem.virt_write(virt_base, &input[..]).unwrap();
    virt_mem.virt_read_into(virt_base, &mut buf[..]).unwrap();
    assert_eq!(buf.len(), input.len());
    assert_eq!(buf, input);
}

#[test]
fn test_virt_read_big_shifted() {
    let mut dummy_mem = DummyMemory::new(size::mb(2));
    let mut buf = vec![0u8; 0x1000 * 16];
    for (i, item) in buf.iter_mut().enumerate() {
        *item = i as u8;
    }
    let (dtb, virt_base) = dummy_mem.alloc_dtb(buf.len().into(), &buf);
    let arch = Architecture::X64;
    let mut virt_mem = VirtualFromPhysical::new(&mut dummy_mem, arch, arch, dtb);

    let mut out = vec![0u8; buf.len() - 0x100];
    virt_mem
        .virt_read_into(virt_base + 0x100, &mut out[..])
        .unwrap();
    assert_eq!(buf[0x100..].to_vec().len(), out.len());
    assert_eq!(buf[0x100..].to_vec(), out);
}

#[test]
fn test_virt_write_big_shifted() {
    let mut dummy_mem = DummyMemory::new(size::mb(2));
    let mut buf = vec![0u8; 0x1000 * 16 - 0x100];
    let mut input = vec![0u8; buf.len()];
    for (i, item) in input.iter_mut().enumerate() {
        *item = i as u8;
    }
    let (dtb, virt_base) = dummy_mem.alloc_dtb(input.len().into(), &input);
    let arch = Architecture::X64;
    let mut virt_mem = VirtualFromPhysical::new(&mut dummy_mem, arch, arch, dtb);

    virt_mem.virt_write(virt_base + 0x100, &input[..]).unwrap();
    virt_mem
        .virt_read_into(virt_base + 0x100, &mut buf[..])
        .unwrap();
    assert_eq!(buf.to_vec().len(), input.len());
    assert_eq!(buf.to_vec(), input);
}
