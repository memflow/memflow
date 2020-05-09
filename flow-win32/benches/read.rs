use std::time::Duration;

#[macro_use]
extern crate bencher;

use bencher::Bencher;

extern crate flow_core;
extern crate flow_qemu_procfs;
extern crate flow_win32;
extern crate rand;

use flow_core::mem::{AccessVirtualMemory, CachedMemoryAccess, TimedCache};
use flow_core::{Length, OsProcess, OsProcessModule, PageType};

use flow_qemu_procfs::Memory;

use flow_win32::{Win32, Win32Module, Win32Offsets, Win32Process};

use rand::prelude::*;
use rand::{prng::XorShiftRng as CurRng, Rng, SeedableRng};

fn rwtest<T: AccessVirtualMemory>(
    mem: &mut T,
    proc: &Win32Process,
    module: &dyn OsProcessModule,
    chunk_sizes: &[usize],
    chunk_counts: &[usize],
    read_size: usize,
) {
    let mut rng = CurRng::from_rng(thread_rng()).unwrap();

    for i in chunk_sizes {
        for o in chunk_counts {
            let mut bufs = vec![(vec![0 as u8; *i], 0); *o];
            let mut done_size = 0;

            while done_size < read_size {
                let base_addr = rng.gen_range(
                    module.base().as_u64(),
                    module.base().as_u64() + module.size().as_u64(),
                );
                for (_, addr) in bufs.iter_mut() {
                    *addr = base_addr + rng.gen_range(0, 0x2000);
                }

                {
                    let mut vmem = proc.virt_mem(mem);
                    for (buf, addr) in bufs.iter_mut() {
                        let _ = vmem.virt_read_raw_into((*addr).into(), buf.as_mut_slice());
                    }
                }
                done_size += *i * *o;
            }
        }
    }
}

fn initialize_ctx() -> flow_core::Result<(Memory, Win32, Win32Process, Win32Module)> {
    let mut mem = Memory::new().unwrap();

    let os = Win32::try_with(&mut mem).unwrap();
    let offsets = Win32Offsets::try_with_guid(&os.kernel_guid()).unwrap();

    let mut rng = CurRng::from_rng(thread_rng()).unwrap();

    let proc_list = os.eprocess_list(&mut mem, &offsets).unwrap();

    for i in -100..(proc_list.len() as isize) {
        let idx = if i >= 0 {
            i as usize
        } else {
            rng.gen_range(0, proc_list.len())
        };

        if let Ok(proc) = Win32Process::try_with_eprocess(&mut mem, &os, &offsets, proc_list[idx]) {
            let mod_list: Vec<Win32Module> = proc
                .peb_list(&mut mem)
                .unwrap_or_default()
                .iter()
                .filter_map(|&x| {
                    if let Ok(module) = Win32Module::try_with_peb(&mut mem, &proc, &offsets, x) {
                        if module.size() > 0x1000.into() {
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
                return Ok((mem, os, proc, tmod.clone()));
            }
        }
    }

    Err("No module found!".into())
}

fn read_test(bench: &mut Bencher, cache_size: u64, chunk_size: usize, chunks: usize) {
    let (mut mem, os, proc, tmod) = initialize_ctx().unwrap();

    if cache_size > 0 {
        let cache = TimedCache::new(
            os.start_block.arch,
            Length::from_mb(cache_size),
            Duration::from_millis(10000).into(),
            PageType::PAGE_TABLE | PageType::READ_ONLY,
        );
        let mut mem_cache = CachedMemoryAccess::with(mem, cache);

        bench.iter(|| {
            rwtest(
                &mut mem_cache,
                &proc,
                &tmod,
                &[chunk_size],
                &[chunks],
                chunk_size,
            );
        });
    } else {
        bench.iter(|| {
            rwtest(&mut mem, &proc, &tmod, &[chunk_size], &[chunks], chunk_size);
        });
    }

    bench.bytes = chunk_size as u64;
}

fn read_0x8_x1(bench: &mut Bencher) {
    read_test(bench, 0, 8, 1);
}

fn read_0x10_x1(bench: &mut Bencher) {
    read_test(bench, 0, 0x10, 1);
}

fn read_0x100_x1(bench: &mut Bencher) {
    read_test(bench, 0, 0x100, 1);
}

fn read_0x1000_x1(bench: &mut Bencher) {
    read_test(bench, 0, 0x1000, 1);
}

fn read_0x10000_x1(bench: &mut Bencher) {
    read_test(bench, 0, 0x10000, 1);
}

benchmark_group!(
    bench_nocache,
    read_0x8_x1,
    read_0x10_x1,
    read_0x100_x1,
    read_0x1000_x1,
    read_0x10000_x1
);

fn read_cache_0x8_x1(bench: &mut Bencher) {
    read_test(bench, 2, 8, 1);
}

fn read_cache_0x10_x1(bench: &mut Bencher) {
    read_test(bench, 2, 0x10, 1);
}

fn read_cache_0x100_x1(bench: &mut Bencher) {
    read_test(bench, 2, 0x100, 1);
}

fn read_cache_0x1000_x1(bench: &mut Bencher) {
    read_test(bench, 2, 0x1000, 1);
}

fn read_cache_0x10000_x1(bench: &mut Bencher) {
    read_test(bench, 2, 0x10000, 1);
}

benchmark_group!(
    bench_cache,
    read_cache_0x8_x1,
    read_cache_0x10_x1,
    read_cache_0x100_x1,
    read_cache_0x1000_x1,
    read_cache_0x10000_x1
);

fn read_size_cache_0x001m_0x8_x1(bench: &mut Bencher) {
    read_test(bench, 1, 0x8, 1);
}

fn read_size_cache_0x002m_0x8_x1(bench: &mut Bencher) {
    read_test(bench, 2, 0x8, 1);
}

fn read_size_cache_0x004m_0x8_x1(bench: &mut Bencher) {
    read_test(bench, 4, 0x8, 1);
}

fn read_size_cache_0x008m_0x8_x1(bench: &mut Bencher) {
    read_test(bench, 8, 0x8, 1);
}

fn read_size_cache_0x010m_0x8_x1(bench: &mut Bencher) {
    read_test(bench, 16, 0x8, 1);
}

fn read_size_cache_0x020m_0x8_x1(bench: &mut Bencher) {
    read_test(bench, 32, 0x8, 1);
}

fn read_size_cache_0x040m_0x8_x1(bench: &mut Bencher) {
    read_test(bench, 64, 0x8, 1);
}

fn read_size_cache_0x080m_0x8_x1(bench: &mut Bencher) {
    read_test(bench, 128, 0x8, 1);
}

fn read_size_cache_0x100m_0x8_x1(bench: &mut Bencher) {
    read_test(bench, 256, 0x8, 1);
}

benchmark_group!(
    bench_size_cache,
    read_size_cache_0x001m_0x8_x1,
    read_size_cache_0x002m_0x8_x1,
    read_size_cache_0x004m_0x8_x1,
    read_size_cache_0x008m_0x8_x1,
    read_size_cache_0x010m_0x8_x1,
    read_size_cache_0x020m_0x8_x1,
    read_size_cache_0x040m_0x8_x1,
    read_size_cache_0x080m_0x8_x1,
    read_size_cache_0x100m_0x8_x1,
);

benchmark_main!(bench_nocache, bench_cache, bench_size_cache);
