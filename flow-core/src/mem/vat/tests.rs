use crate::architecture::Architecture;
use crate::dummy::DummyMemory;
use crate::mem::cache::page_cache::PageCache;
use crate::mem::cache::timed_validator::TimedCacheValidator;
use crate::mem::{VirtualAdressTranslator, VirtualFromPhysical, VirtualMemory, VAT};
use crate::types::{Address, Length, PhysicalAddress};
use crate::*;

use rand::{thread_rng, Rng};

#[test]
fn test_cached_mem() {
    let mut dummy_mem = DummyMemory::new(Length::from_mb(512));

    let virt_size = Length::from_mb(8);
    let mut test_buf = vec![0_u64; virt_size.as_usize() / 8];

    for i in &mut test_buf {
        *i = thread_rng().gen::<u64>();
    }

    let test_buf =
        unsafe { std::slice::from_raw_parts(test_buf.as_ptr() as *const u8, virt_size.as_usize()) };

    let (dtb, virt_base) = dummy_mem.alloc_dtb(virt_size, &test_buf);
    let arch = Architecture::X64;

    let mut buf_nocache = vec![0_u8; test_buf.len()];
    {
        let mut virt_mem = VirtualFromPhysical::new(&mut dummy_mem, arch, arch, dtb);
        virt_mem
            .virt_read_raw_into(virt_base, buf_nocache.as_mut_slice())
            .unwrap();
    }

    assert_eq!(buf_nocache, test_buf);

    let cache = PageCache::new(
        arch,
        Length::from_mb(2),
        PageType::PAGE_TABLE | PageType::READ_ONLY,
        TimedCacheValidator::new(coarsetime::Duration::from_millis(100)),
    );
    let mut mem_cache = CachedMemoryAccess::with(&mut dummy_mem, cache);
    let mut buf_cache = vec![0_u8; buf_nocache.len()];
    {
        let mut virt_mem = VirtualFromPhysical::new(&mut mem_cache, arch, arch, dtb);
        virt_mem
            .virt_read_raw_into(virt_base, buf_cache.as_mut_slice())
            .unwrap();
    }

    assert_eq!(buf_nocache, buf_cache);
}

#[test]
fn test_cache_invalidity_cached() {
    let mut dummy_mem = DummyMemory::new(Length::from_mb(512));
    let mem_ptr = &mut dummy_mem as *mut DummyMemory;
    let virt_size = Length::from_mb(8);
    let mut buf_start = vec![0_u8; 64];
    for (i, item) in buf_start.iter_mut().enumerate() {
        *item = (i % 256) as u8;
    }
    let (dtb, virt_base) = dummy_mem.alloc_dtb(virt_size, &buf_start);
    let arch = Architecture::X64;

    let cache = PageCache::new(
        arch,
        Length::from_mb(2),
        PageType::PAGE_TABLE | PageType::READ_ONLY | PageType::WRITEABLE,
        TimedCacheValidator::new(coarsetime::Duration::from_secs(100)),
    );

    let mut mem_cache = CachedMemoryAccess::with(&mut dummy_mem, cache);

    //Modifying the memory from other channels should leave the cached page unchanged
    let mut cached_buf = vec![0_u8; 64];
    {
        let mut virt_mem = VirtualFromPhysical::new(&mut mem_cache, arch, arch, dtb);
        virt_mem
            .virt_read_raw_into(virt_base, cached_buf.as_mut_slice())
            .unwrap();
    }

    let mut write_buf = cached_buf.clone();
    write_buf[16..20].copy_from_slice(&[255, 255, 255, 255]);
    {
        let mut virt_mem =
            VirtualFromPhysical::new(unsafe { mem_ptr.as_mut().unwrap() }, arch, arch, dtb);
        virt_mem
            .virt_write_raw(virt_base, write_buf.as_slice())
            .unwrap();
    }

    let mut check_buf = vec![0_u8; 64];
    {
        let mut virt_mem = VirtualFromPhysical::new(&mut mem_cache, arch, arch, dtb);
        virt_mem
            .virt_read_raw_into(virt_base, check_buf.as_mut_slice())
            .unwrap();
    }

    assert_eq!(cached_buf, check_buf);
    assert_ne!(check_buf, write_buf);
}

#[test]
fn test_cache_invalidity_non_cached() {
    let mut dummy_mem = DummyMemory::new(Length::from_mb(512));
    let mem_ptr = &mut dummy_mem as *mut DummyMemory;
    let virt_size = Length::from_mb(8);
    let mut buf_start = vec![0_u8; 64];
    for (i, item) in buf_start.iter_mut().enumerate() {
        *item = (i % 256) as u8;
    }
    let (dtb, virt_base) = dummy_mem.alloc_dtb(virt_size, &buf_start);
    let arch = Architecture::X64;

    //alloc_dtb creates a page table with all writeable pages, we disable cache for them
    let cache = PageCache::new(
        arch,
        Length::from_mb(2),
        PageType::PAGE_TABLE | PageType::READ_ONLY,
        TimedCacheValidator::new(coarsetime::Duration::from_secs(100)),
    );

    let mut mem_cache = CachedMemoryAccess::with(&mut dummy_mem, cache);

    //Modifying the memory from other channels should leave the cached page unchanged
    let mut cached_buf = vec![0_u8; 64];
    {
        let mut virt_mem = VirtualFromPhysical::new(&mut mem_cache, arch, arch, dtb);
        virt_mem
            .virt_read_raw_into(virt_base, cached_buf.as_mut_slice())
            .unwrap();
    }

    let mut write_buf = cached_buf.clone();
    write_buf[16..20].copy_from_slice(&[255, 255, 255, 255]);
    {
        let mut virt_mem =
            VirtualFromPhysical::new(unsafe { mem_ptr.as_mut().unwrap() }, arch, arch, dtb);
        virt_mem
            .virt_write_raw(virt_base, write_buf.as_slice())
            .unwrap();
    }

    let mut check_buf = vec![0_u8; 64];
    {
        let mut virt_mem = VirtualFromPhysical::new(mem_cache, arch, arch, dtb);
        virt_mem
            .virt_read_raw_into(virt_base, check_buf.as_mut_slice())
            .unwrap();
    }

    assert_ne!(cached_buf, check_buf);
    assert_eq!(check_buf, write_buf);
}

#[test]
fn test_cache_phys_mem() {
    let mut dummy_mem = DummyMemory::new(Length::from_mb(16));

    let mut buf_start = vec![0_u8; 64];
    for (i, item) in buf_start.iter_mut().enumerate() {
        *item = (i % 256) as u8;
    }

    let address = Address::from(0x5323);

    let addr = PhysicalAddress::with_page(
        address,
        crate::types::PageType::from_writeable_bit(false),
        0x1000.into(),
    );

    dummy_mem
        .phys_write_raw(addr, buf_start.as_slice())
        .unwrap();

    let arch = Architecture::X64;

    let cache = PageCache::new(
        arch,
        Length::from_mb(2),
        PageType::PAGE_TABLE | PageType::READ_ONLY,
        TimedCacheValidator::new(coarsetime::Duration::from_secs(100)),
    );

    let mut mem_cache = CachedMemoryAccess::with(&mut dummy_mem, cache);

    let mut buf_1 = vec![0_u8; 64];
    mem_cache
        .phys_read_into(addr, buf_1.as_mut_slice())
        .unwrap();

    assert_eq!(buf_start, buf_1);
}

#[test]
fn test_writeback() {
    let mut dummy_mem = DummyMemory::new(Length::from_mb(16));
    let virt_size = Length::from_mb(8);
    let mut buf_start = vec![0_u8; 64];
    for (i, item) in buf_start.iter_mut().enumerate() {
        *item = (i % 256) as u8;
    }
    let (dtb, virt_base) = dummy_mem.alloc_dtb(virt_size, &buf_start);
    let arch = Architecture::X64;

    let cache = PageCache::new(
        arch,
        Length::from_mb(2),
        PageType::PAGE_TABLE | PageType::READ_ONLY,
        TimedCacheValidator::new(coarsetime::Duration::from_secs(100)),
    );

    let mut mem_cache = CachedMemoryAccess::with(&mut dummy_mem, cache);
    let mut virt_mem = VirtualFromPhysical::new(&mut mem_cache, arch, arch, dtb);

    let mut buf_1 = vec![0_u8; 64];
    virt_mem
        .virt_read_into(virt_base, buf_1.as_mut_slice())
        .unwrap();

    assert_eq!(buf_start, buf_1);
    buf_1[16..20].copy_from_slice(&[255, 255, 255, 255]);
    virt_mem
        .virt_write(virt_base + Length::from(16), &buf_1[16..20])
        .unwrap();

    let mut buf_2 = vec![0_u8; 64];
    virt_mem
        .virt_read_into(virt_base, buf_2.as_mut_slice())
        .unwrap();

    assert_eq!(buf_1, buf_2);
    assert_ne!(buf_2, buf_start);

    let mut buf_3 = vec![0_u8; 64];

    virt_mem
        .virt_read_into(virt_base, buf_3.as_mut_slice())
        .unwrap();
    assert_eq!(buf_2, buf_3);
}

#[test]
fn test_vtop() {
    let mut dummy_mem = DummyMemory::new(Length::from_mb(512));
    let virt_size = Length::from_mb(8);
    let (dtb, virt_base) = dummy_mem.alloc_dtb(virt_size, &[]);
    let arch = Architecture::X64;
    let mut vat = VirtualAdressTranslator::new(arch);

    assert_eq!(
        vat.virt_to_phys(&mut dummy_mem, dtb, virt_base).is_ok(),
        true
    );
    assert_eq!(
        arch.virt_to_phys(
            &mut dummy_mem,
            dtb,
            virt_base + Length::from(virt_size.as_usize() / 2),
        )
        .is_ok(),
        true
    );
    assert_eq!(
        vat.virt_to_phys(&mut dummy_mem, dtb, virt_base - Length::from_mb(1))
            .is_ok(),
        false
    );
    assert_eq!(
        vat.virt_to_phys(&mut dummy_mem, dtb, virt_base + virt_size)
            .is_ok(),
        false
    );
}

#[test]
fn test_virt_read_small() {
    let mut dummy_mem = DummyMemory::new(Length::from_mb(2));
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
    let mut dummy_mem = DummyMemory::new(Length::from_mb(2));
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
    let mut dummy_mem = DummyMemory::new(Length::from_mb(2));
    let mut buf = vec![0u8; 256];
    for (i, item) in buf.iter_mut().enumerate() {
        *item = i as u8;
    }
    let (dtb, virt_base) = dummy_mem.alloc_dtb(buf.len().into(), &buf);
    let arch = Architecture::X64;
    let mut virt_mem = VirtualFromPhysical::new(&mut dummy_mem, arch, arch, dtb);

    let mut out = vec![0u8; buf.len() - 128];
    virt_mem
        .virt_read_into(virt_base + Length::from(128), &mut out[..])
        .unwrap();
    assert_eq!(buf[128..].to_vec().len(), out.len());
    assert_eq!(buf[128..].to_vec(), out);
}

#[test]
fn test_virt_write_small_shifted() {
    let mut dummy_mem = DummyMemory::new(Length::from_mb(2));
    let mut buf = vec![0u8; 128];
    let mut input = vec![0u8; buf.len()];
    for (i, item) in input.iter_mut().enumerate() {
        *item = i as u8;
    }
    let (dtb, virt_base) = dummy_mem.alloc_dtb(input.len().into(), &input);
    let arch = Architecture::X64;
    let mut virt_mem = VirtualFromPhysical::new(&mut dummy_mem, arch, arch, dtb);

    virt_mem
        .virt_write(virt_base + Length::from(128), &input[..])
        .unwrap();
    virt_mem
        .virt_read_into(virt_base + Length::from(128), &mut buf[..])
        .unwrap();
    assert_eq!(buf.to_vec().len(), input.len());
    assert_eq!(buf.to_vec(), input);
}

#[test]
fn test_virt_read_medium() {
    let mut dummy_mem = DummyMemory::new(Length::from_mb(2));
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
    let mut dummy_mem = DummyMemory::new(Length::from_mb(2));
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
    let mut dummy_mem = DummyMemory::new(Length::from_mb(2));
    let mut buf = vec![0u8; 0x1000];
    for (i, item) in buf.iter_mut().enumerate() {
        *item = i as u8;
    }
    let (dtb, virt_base) = dummy_mem.alloc_dtb(buf.len().into(), &buf);
    let arch = Architecture::X64;
    let mut virt_mem = VirtualFromPhysical::new(&mut dummy_mem, arch, arch, dtb);

    let mut out = vec![0u8; buf.len() - 0x100];
    virt_mem
        .virt_read_into(virt_base + Length::from(0x100), &mut out[..])
        .unwrap();
    assert_eq!(buf[0x100..].to_vec().len(), out.len());
    assert_eq!(buf[0x100..].to_vec(), out);
}

#[test]
fn test_virt_write_medium_shifted() {
    let mut dummy_mem = DummyMemory::new(Length::from_mb(2));
    let mut buf = vec![0u8; 0x1000 - 0x100];
    let mut input = vec![0u8; buf.len()];
    for (i, item) in input.iter_mut().enumerate() {
        *item = i as u8;
    }
    let (dtb, virt_base) = dummy_mem.alloc_dtb(input.len().into(), &input);
    let arch = Architecture::X64;
    let mut virt_mem = VirtualFromPhysical::new(&mut dummy_mem, arch, arch, dtb);

    virt_mem
        .virt_write(virt_base + Length::from(0x100), &input[..])
        .unwrap();
    virt_mem
        .virt_read_into(virt_base + Length::from(0x100), &mut buf[..])
        .unwrap();
    assert_eq!(buf.to_vec().len(), input.len());
    assert_eq!(buf.to_vec(), input);
}

#[test]
fn test_virt_read_big() {
    let mut dummy_mem = DummyMemory::new(Length::from_mb(2));
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
    let mut dummy_mem = DummyMemory::new(Length::from_mb(2));
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
    let mut dummy_mem = DummyMemory::new(Length::from_mb(2));
    let mut buf = vec![0u8; 0x1000 * 16];
    for (i, item) in buf.iter_mut().enumerate() {
        *item = i as u8;
    }
    let (dtb, virt_base) = dummy_mem.alloc_dtb(buf.len().into(), &buf);
    let arch = Architecture::X64;
    let mut virt_mem = VirtualFromPhysical::new(&mut dummy_mem, arch, arch, dtb);

    let mut out = vec![0u8; buf.len() - 0x100];
    virt_mem
        .virt_read_into(virt_base + Length::from(0x100), &mut out[..])
        .unwrap();
    assert_eq!(buf[0x100..].to_vec().len(), out.len());
    assert_eq!(buf[0x100..].to_vec(), out);
}

#[test]
fn test_virt_write_big_shifted() {
    let mut dummy_mem = DummyMemory::new(Length::from_mb(2));
    let mut buf = vec![0u8; 0x1000 * 16 - 0x100];
    let mut input = vec![0u8; buf.len()];
    for (i, item) in input.iter_mut().enumerate() {
        *item = i as u8;
    }
    let (dtb, virt_base) = dummy_mem.alloc_dtb(input.len().into(), &input);
    let arch = Architecture::X64;
    let mut virt_mem = VirtualFromPhysical::new(&mut dummy_mem, arch, arch, dtb);

    virt_mem
        .virt_write(virt_base + Length::from(0x100), &input[..])
        .unwrap();
    virt_mem
        .virt_read_into(virt_base + Length::from(0x100), &mut buf[..])
        .unwrap();
    assert_eq!(buf.to_vec().len(), input.len());
    assert_eq!(buf.to_vec(), input);
}
