use crate::architecture::x86::x64;
use crate::cglue::ForwardMut;
use crate::dummy::{DummyMemory, DummyOs};
use crate::mem::{
    DirectTranslate, MemoryView, PhysicalMemory, VirtualDma, VirtualTranslate, VirtualTranslate2,
    VirtualTranslate3,
};
use crate::types::{mem, size, PageType};
use cglue::tuple::*;

#[test]
fn test_vtop() {
    let dummy_mem = DummyMemory::new(size::mb(32));
    let mut dummy_os = DummyOs::new(dummy_mem);
    let virt_size = size::mb(8);
    let (dtb, virt_base) = dummy_os.alloc_dtb(virt_size, &[]);
    let translator = x64::new_translator(dtb);
    let mut vat = DirectTranslate::new();

    for i in (0..virt_size).step_by(128) {
        let virt_base = virt_base + i;
        let vtop = match vat.virt_to_phys(dummy_os.as_mut(), &translator, virt_base) {
            Err(_) => None,
            Ok(paddr) => Some(paddr.address()),
        };
        let dummy_vtop = dummy_os.vtop(dtb, virt_base);

        assert_eq!(vtop, dummy_vtop);
    }

    for i in 0_u64..128 {
        let virt_base = virt_base + virt_size + i;
        let vtop = match vat.virt_to_phys(dummy_os.as_mut(), &translator, virt_base) {
            Err(_) => None,
            Ok(paddr) => Some(paddr.address()),
        };
        let dummy_vtop = dummy_os.vtop(dtb, virt_base);

        assert!(vtop.is_none());

        assert_eq!(vtop, dummy_vtop);
    }

    for i in 0..128 {
        let virt_base = virt_base - i;
        let vtop = match vat.virt_to_phys(dummy_os.as_mut(), &translator, virt_base) {
            Err(_) => None,
            Ok(paddr) => Some(paddr.address()),
        };
        let dummy_vtop = dummy_os.vtop(dtb, virt_base);

        assert!(i == 0 || vtop.is_none());

        assert_eq!(vtop, dummy_vtop);
    }
}

#[test]
fn test_x86_flag_inheritance() {
    let dummy_mem = DummyMemory::new(size::mb(16));
    let mut dummy_os = DummyOs::new(dummy_mem);
    let (dtb, virt_base) = dummy_os.alloc_dtb(size::mb(2), &[]);
    let translator = x64::new_translator(dtb);

    let paddr = translator.virt_to_phys(&mut dummy_os, virt_base).unwrap();
    assert!(!paddr.page_type.contains(PageType::NOEXEC));

    {
        let mut phys_view = dummy_os.phys_view();
        let dtb_idx = (virt_base.to_umem() as u64 >> 39) & 0x1ffu64;
        let pte = phys_view.read_addr64(dtb + dtb_idx * 8).unwrap().to_umem();
        // Set nx bit
        let pte = pte | !(!0u64 >> 1);
        phys_view.write(dtb + dtb_idx * 8, &pte).unwrap();
    }

    let paddr = translator.virt_to_phys(&mut dummy_os, virt_base).unwrap();
    assert!(paddr.page_type.contains(PageType::NOEXEC));
}

#[test]
fn test_virt_page_map() {
    let dummy_mem = DummyMemory::new(size::mb(16));
    let mut dummy_os = DummyOs::new(dummy_mem);
    let (dtb, virt_base) = dummy_os.alloc_dtb(size::mb(2), &[]);
    let translator = x64::new_translator(dtb);
    let arch = x64::ARCH;
    let mut virt_mem = VirtualDma::new(dummy_os.forward_mut(), arch, translator);

    let page_map = virt_mem.virt_page_map_vec(0);

    for CTup3(address, size, pt) in page_map.iter() {
        println!("{:x}-{:x} ({:x}) {:?}", address, *address + *size, size, pt);
    }

    assert!(page_map.len() == 1);
    assert_eq!(page_map[0].0, virt_base);
    assert_eq!(page_map[0].1, mem::mb(2));
}

#[test]
fn test_virt_read_small() {
    let dummy_mem = DummyMemory::new(size::mb(2));
    let mut dummy_os = DummyOs::new(dummy_mem);
    let mut buf = vec![0u8; 256];
    for (i, item) in buf.iter_mut().enumerate() {
        *item = i as u8;
    }
    let (dtb, virt_base) = dummy_os.alloc_dtb(buf.len(), &buf);
    let translator = x64::new_translator(dtb);
    let arch = x64::ARCH;
    let mut virt_mem = VirtualDma::new(dummy_os.forward_mut(), arch, translator);

    let mut out = vec![0u8; buf.len()];
    virt_mem.read_into(virt_base, &mut out[..]).unwrap();
    assert_eq!(buf.len(), out.len());
    assert_eq!(buf, out);
}

#[test]
fn test_virt_write_small() {
    let dummy_mem = DummyMemory::new(size::mb(2));
    let mut dummy_os = DummyOs::new(dummy_mem);
    let mut buf = vec![0u8; 256];
    let mut input = vec![0u8; buf.len()];
    for (i, item) in input.iter_mut().enumerate() {
        *item = i as u8;
    }
    let (dtb, virt_base) = dummy_os.alloc_dtb(input.len(), &input);
    let translator = x64::new_translator(dtb);
    let arch = x64::ARCH;
    let mut virt_mem = VirtualDma::new(dummy_os.forward_mut(), arch, translator);

    virt_mem.write(virt_base, &input[..]).unwrap();
    virt_mem.read_into(virt_base, &mut buf[..]).unwrap();
    assert_eq!(buf.len(), input.len());
    assert_eq!(buf, input);
}

#[test]
fn test_virt_read_small_shifted() {
    let dummy_mem = DummyMemory::new(size::mb(2));
    let mut dummy_os = DummyOs::new(dummy_mem);
    let mut buf = vec![0u8; 256];
    for (i, item) in buf.iter_mut().enumerate() {
        *item = i as u8;
    }
    let (dtb, virt_base) = dummy_os.alloc_dtb(buf.len(), &buf);
    let translator = x64::new_translator(dtb);
    let arch = x64::ARCH;
    let mut virt_mem = VirtualDma::new(dummy_os.forward_mut(), arch, translator);

    let mut out = vec![0u8; buf.len() - 128];
    virt_mem.read_into(virt_base + 128, &mut out[..]).unwrap();
    assert_eq!(buf[128..].to_vec().len(), out.len());
    assert_eq!(buf[128..].to_vec(), out);
}

#[test]
fn test_virt_write_small_shifted() {
    let dummy_mem = DummyMemory::new(size::mb(2));
    let mut dummy_os = DummyOs::new(dummy_mem);
    let mut buf = [0u8; 128];
    let mut input = vec![0u8; buf.len()];
    for (i, item) in input.iter_mut().enumerate() {
        *item = i as u8;
    }
    let (dtb, virt_base) = dummy_os.alloc_dtb(input.len(), &input);
    let translator = x64::new_translator(dtb);
    let arch = x64::ARCH;
    let mut virt_mem = VirtualDma::new(dummy_os.forward_mut(), arch, translator);

    virt_mem.write(virt_base + 128, &input[..]).unwrap();
    virt_mem.read_into(virt_base + 128, &mut buf[..]).unwrap();
    assert_eq!(buf.to_vec().len(), input.len());
    assert_eq!(buf.to_vec(), input);
}

#[test]
fn test_virt_read_medium() {
    let dummy_mem = DummyMemory::new(size::mb(2));
    let mut dummy_os = DummyOs::new(dummy_mem);
    let mut buf = vec![0u8; 0x1000];
    for (i, item) in buf.iter_mut().enumerate() {
        *item = i as u8;
    }
    let (dtb, virt_base) = dummy_os.alloc_dtb(buf.len(), &buf);
    let translator = x64::new_translator(dtb);
    let arch = x64::ARCH;
    let mut virt_mem = VirtualDma::new(dummy_os.forward_mut(), arch, translator);

    let mut out = vec![0u8; buf.len()];
    virt_mem.read_into(virt_base, &mut out[..]).unwrap();
    assert_eq!(buf.len(), out.len());
    assert_eq!(buf, out);
}

#[test]
fn test_virt_write_medium() {
    let dummy_mem = DummyMemory::new(size::mb(2));
    let mut dummy_os = DummyOs::new(dummy_mem);
    let mut buf = vec![0u8; 0x1000];
    let mut input = vec![0u8; buf.len()];
    for (i, item) in input.iter_mut().enumerate() {
        *item = i as u8;
    }
    let (dtb, virt_base) = dummy_os.alloc_dtb(input.len(), &input);
    let translator = x64::new_translator(dtb);
    let arch = x64::ARCH;
    let mut virt_mem = VirtualDma::new(dummy_os.forward_mut(), arch, translator);

    virt_mem.write(virt_base, &input[..]).unwrap();
    virt_mem.read_into(virt_base, &mut buf[..]).unwrap();
    assert_eq!(buf.len(), input.len());
    assert_eq!(buf, input);
}

#[test]
fn test_virt_read_medium_shifted() {
    let dummy_mem = DummyMemory::new(size::mb(2));
    let mut dummy_os = DummyOs::new(dummy_mem);
    let mut buf = vec![0u8; 0x1000];
    for (i, item) in buf.iter_mut().enumerate() {
        *item = i as u8;
    }
    let (dtb, virt_base) = dummy_os.alloc_dtb(buf.len(), &buf);
    let translator = x64::new_translator(dtb);
    let arch = x64::ARCH;
    let mut virt_mem = VirtualDma::new(dummy_os.forward_mut(), arch, translator);

    let mut out = vec![0u8; buf.len() - 0x100];
    virt_mem.read_into(virt_base + 0x100, &mut out[..]).unwrap();
    assert_eq!(buf[0x100..].to_vec().len(), out.len());
    assert_eq!(buf[0x100..].to_vec(), out);
}

#[test]
fn test_virt_write_medium_shifted() {
    let dummy_mem = DummyMemory::new(size::mb(2));
    let mut dummy_os = DummyOs::new(dummy_mem);
    let mut buf = vec![0u8; 0x1000 - 0x100];
    let mut input = vec![0u8; buf.len()];
    for (i, item) in input.iter_mut().enumerate() {
        *item = i as u8;
    }
    let (dtb, virt_base) = dummy_os.alloc_dtb(input.len(), &input);
    let translator = x64::new_translator(dtb);
    let arch = x64::ARCH;
    let mut virt_mem = VirtualDma::new(dummy_os.forward_mut(), arch, translator);

    virt_mem.write(virt_base + 0x100, &input[..]).unwrap();
    virt_mem.read_into(virt_base + 0x100, &mut buf[..]).unwrap();
    assert_eq!(buf.to_vec().len(), input.len());
    assert_eq!(buf.to_vec(), input);
}

#[test]
fn test_virt_read_big() {
    let dummy_mem = DummyMemory::new(size::mb(2));
    let mut dummy_os = DummyOs::new(dummy_mem);
    let mut buf = vec![0u8; 0x1000 * 16];
    for (i, item) in buf.iter_mut().enumerate() {
        *item = i as u8;
    }
    let (dtb, virt_base) = dummy_os.alloc_dtb(buf.len(), &buf);
    let translator = x64::new_translator(dtb);
    let arch = x64::ARCH;
    let mut virt_mem = VirtualDma::new(dummy_os.forward_mut(), arch, translator);

    let mut out = vec![0u8; buf.len()];
    virt_mem.read_into(virt_base, &mut out[..]).unwrap();
    assert_eq!(buf.len(), out.len());
    assert_eq!(buf, out);
}

#[test]
fn test_virt_write_big() {
    let dummy_mem = DummyMemory::new(size::mb(2));
    let mut dummy_os = DummyOs::new(dummy_mem);
    let mut buf = vec![0u8; 0x1000 * 16];
    let mut input = vec![0u8; buf.len()];
    for (i, item) in input.iter_mut().enumerate() {
        *item = i as u8;
    }
    let (dtb, virt_base) = dummy_os.alloc_dtb(input.len(), &input);
    let translator = x64::new_translator(dtb);
    let arch = x64::ARCH;
    let mut virt_mem = VirtualDma::new(dummy_os.forward_mut(), arch, translator);

    virt_mem.write(virt_base, &input[..]).unwrap();
    virt_mem.read_into(virt_base, &mut buf[..]).unwrap();
    assert_eq!(buf.len(), input.len());
    assert_eq!(buf, input);
}

#[test]
fn test_virt_read_big_shifted() {
    let dummy_mem = DummyMemory::new(size::mb(2));
    let mut dummy_os = DummyOs::new(dummy_mem);
    let mut buf = vec![0u8; 0x1000 * 16];
    for (i, item) in buf.iter_mut().enumerate() {
        *item = i as u8;
    }
    let (dtb, virt_base) = dummy_os.alloc_dtb(buf.len(), &buf);
    let translator = x64::new_translator(dtb);
    let arch = x64::ARCH;
    let mut virt_mem = VirtualDma::new(dummy_os.forward_mut(), arch, translator);

    let mut out = vec![0u8; buf.len() - 0x100];
    virt_mem.read_into(virt_base + 0x100, &mut out[..]).unwrap();
    assert_eq!(buf[0x100..].to_vec().len(), out.len());
    assert_eq!(buf[0x100..].to_vec(), out);
}

#[test]
fn test_virt_write_big_shifted() {
    let dummy_mem = DummyMemory::new(size::mb(2));
    let mut dummy_os = DummyOs::new(dummy_mem);
    let mut buf = vec![0u8; 0x1000 * 16 - 0x100];
    let mut input = vec![0u8; buf.len()];
    for (i, item) in input.iter_mut().enumerate() {
        *item = i as u8;
    }
    let (dtb, virt_base) = dummy_os.alloc_dtb(input.len(), &input);
    let translator = x64::new_translator(dtb);
    let arch = x64::ARCH;
    let mut virt_mem = VirtualDma::new(dummy_os.forward_mut(), arch, translator);

    virt_mem.write(virt_base + 0x100, &input[..]).unwrap();
    virt_mem.read_into(virt_base + 0x100, &mut buf[..]).unwrap();
    assert_eq!(buf.to_vec().len(), input.len());
    assert_eq!(buf.to_vec(), input);
}
