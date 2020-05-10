use std::time::Duration;

#[macro_use]
extern crate bencher;

use bencher::{black_box, Bencher};

extern crate flow_core;
extern crate flow_qemu_procfs;
extern crate flow_win32;
extern crate rand;

use flow_core::arch::Architecture;
use flow_core::mem::{
    AccessPhysicalMemory, AccessVirtualMemory, CachedMemoryAccess, CachedVAT, TimedCache, TimedTLB,
    VirtualAddressTranslator,
};
use flow_core::types::Address;

use flow_qemu_procfs::Memory;

use flow_core::{Length, OsProcess, OsProcessModule, PageType};
use flow_win32::{Win32, Win32Module, Win32Offsets, Win32Process};

use rand::prelude::*;
use rand::{prng::XorShiftRng as CurRng, Rng, SeedableRng};

fn find_module<T: AccessPhysicalMemory + AccessVirtualMemory>(
    mem: &mut T,
) -> flow_core::Result<(Win32, Win32Process, Win32Module)> {
    let os = Win32::try_with(mem)?;
    let offsets = Win32Offsets::try_with_guid(&os.kernel_guid())?;

    let mut rng = CurRng::from_rng(thread_rng()).unwrap();

    let proc_list = os.eprocess_list(mem, &offsets).unwrap();

    for i in -100..(proc_list.len() as isize) {
        let idx = if i >= 0 {
            i as usize
        } else {
            rng.gen_range(0, proc_list.len())
        };

        if let Ok(proc) = Win32Process::try_with_eprocess(mem, &os, &offsets, proc_list[idx]) {
            let mod_list: Vec<Win32Module> = proc
                .peb_list(mem)
                .unwrap_or_default()
                .iter()
                .filter_map(|&x| {
                    if let Ok(module) = Win32Module::try_with_peb(mem, &proc, &offsets, x) {
                        if module.size() > 0x10000.into() {
                            Some(module)
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                })
                .collect();

            if !mod_list.is_empty() {
                let tmod = &mod_list[rng.gen_range(0, mod_list.len())];
                return Ok((os, proc, tmod.clone()));
            }
        }
    }

    Err("No module found!".into())
}

#[allow(clippy::too_many_arguments)]
fn vat_test<T: AccessVirtualMemory + AccessPhysicalMemory + VirtualAddressTranslator>(
    bench: &mut Bencher,
    mut mem: T,
    range_start: u64,
    range_end: u64,
    translations: usize,
    dtb: Address,
    arch: Architecture,
    use_tlb: bool,
) {
    let mut rng = CurRng::from_rng(thread_rng()).unwrap();

    if use_tlb {
        let tlb = TimedTLB::new(2048.into(), Duration::from_millis(1000).into());
        let mut mem = CachedVAT::with(mem, tlb);
        bench.iter(|| {
            for _ in 0..translations {
                let vaddr: Address = rng.gen_range(range_start, range_end).into();
                let _thing = mem.virt_to_phys(arch, dtb, vaddr);
            }
        });
    } else {
        bench.iter(|| {
            for _ in black_box(0..translations) {
                let vaddr: Address = rng.gen_range(range_start, range_end).into();
                let _thing = mem.virt_to_phys(arch, dtb, vaddr);
            }
        });
    }

    bench.bytes = (translations * 8) as u64;
}

fn translate_range(bench: &mut Bencher, range_start: u64, range_end: u64, use_tlb: bool) {
    let mut mem_sys = Memory::new().unwrap();
    let (os, proc, _) = find_module(&mut mem_sys).unwrap();
    let cache = TimedCache::new(
        os.start_block.arch,
        Length::from_mb(32),
        Duration::from_millis(1000).into(),
        PageType::PAGE_TABLE | PageType::READ_ONLY,
    );
    let mem = CachedMemoryAccess::with(mem_sys, cache);
    vat_test(
        bench,
        mem,
        range_start,
        range_end,
        0x100,
        proc.dtb(),
        proc.sys_arch(),
        use_tlb,
    );
}

fn translate_notlb_module(bench: &mut Bencher) {
    let mut mem_sys = Memory::new().unwrap();
    let (os, proc, tmod) = find_module(&mut mem_sys).unwrap();
    let cache = TimedCache::new(
        os.start_block.arch,
        Length::from_mb(2),
        Duration::from_millis(1000).into(),
        PageType::PAGE_TABLE | PageType::READ_ONLY,
    );
    let mem = CachedMemoryAccess::with(mem_sys, cache);
    vat_test(
        bench,
        mem,
        tmod.base().as_u64(),
        tmod.base().as_u64() + tmod.size().as_u64(),
        0x100,
        proc.dtb(),
        proc.sys_arch(),
        false,
    );
}

fn translate_notlb_module_smallrange(bench: &mut Bencher) {
    let mut mem_sys = Memory::new().unwrap();
    let (os, proc, tmod) = find_module(&mut mem_sys).unwrap();
    let cache = TimedCache::new(
        os.start_block.arch,
        Length::from_mb(2),
        Duration::from_millis(1000).into(),
        PageType::PAGE_TABLE | PageType::READ_ONLY,
    );
    let mem = CachedMemoryAccess::with(mem_sys, cache);
    vat_test(
        bench,
        mem,
        tmod.base().as_u64(),
        tmod.base().as_u64() + 0x2000,
        0x100,
        proc.dtb(),
        proc.sys_arch(),
        false,
    );
}

fn translate_notlb_allmem(bench: &mut Bencher) {
    translate_range(bench, 0, !0, false);
}

benchmark_group!(
    translate_notlb,
    translate_notlb_module,
    translate_notlb_module_smallrange,
    translate_notlb_allmem
);

fn translate_tlb_module(bench: &mut Bencher) {
    let mut mem_sys = Memory::new().unwrap();
    let (os, proc, tmod) = find_module(&mut mem_sys).unwrap();
    let cache = TimedCache::new(
        os.start_block.arch,
        Length::from_mb(2),
        Duration::from_millis(1000).into(),
        PageType::PAGE_TABLE | PageType::READ_ONLY,
    );
    let mem = CachedMemoryAccess::with(mem_sys, cache);
    vat_test(
        bench,
        mem,
        tmod.base().as_u64(),
        tmod.base().as_u64() + tmod.size().as_u64(),
        0x100,
        proc.dtb(),
        proc.sys_arch(),
        true,
    );
}

fn translate_tlb_module_smallrange(bench: &mut Bencher) {
    let mut mem_sys = Memory::new().unwrap();
    let (os, proc, tmod) = find_module(&mut mem_sys).unwrap();
    let cache = TimedCache::new(
        os.start_block.arch,
        Length::from_mb(2),
        Duration::from_millis(1000).into(),
        PageType::PAGE_TABLE | PageType::READ_ONLY,
    );
    let mem = CachedMemoryAccess::with(mem_sys, cache);
    vat_test(
        bench,
        mem,
        tmod.base().as_u64(),
        tmod.base().as_u64() + 0x2000,
        0x100,
        proc.dtb(),
        proc.sys_arch(),
        true,
    );
}

fn translate_tlb_allmem(bench: &mut Bencher) {
    translate_range(bench, 0, !0, true);
}

benchmark_group!(
    translate_tlb,
    translate_tlb_module,
    translate_tlb_module_smallrange,
    translate_tlb_allmem
);

benchmark_main!(translate_notlb, translate_tlb);
