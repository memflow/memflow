#[macro_use]
extern crate bencher;

use bencher::Bencher;

extern crate flow_core;
extern crate flow_qemu_procfs;
extern crate flow_win32;
extern crate rand;

use flow_core::mem::{PageType, TimedCache};
use flow_core::{Length, OsProcess, OsProcessModule};

use flow_qemu_procfs::Memory;

use flow_win32::{Win32, Win32Module, Win32Offsets, Win32Process};

use rand::prelude::*;
use rand::{prng::XorShiftRng as CurRng, Rng, SeedableRng};

fn rwtest(
    mem: &mut Memory<TimedCache>,
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

fn initialize_ctx() -> flow_core::Result<(Memory<TimedCache>, Win32Process, Win32Module)> {
    let mut mem = Memory::new(TimedCache::new(
        1000,
        0x200,
        Length::from_kb(4),
        PageType::PAGE_TABLE | PageType::READ_ONLY,
    ))
    .unwrap();
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
                return Ok((mem, proc, tmod.clone()));
            }
        }
    }

    Err("No module found!".into())
}

fn read_test(bench: &mut Bencher, chunk_size: usize, chunks: usize) {
    let (mut mem, proc, tmod) = initialize_ctx().unwrap();

    bench.iter(|| {
        rwtest(&mut mem, &proc, &tmod, &[chunk_size], &[chunks], chunk_size);
    });

    bench.bytes = chunk_size as u64;
}

fn read_0x8_x1(bench: &mut Bencher) {
    read_test(bench, 8, 1);
}

fn read_0x10_x1(bench: &mut Bencher) {
    read_test(bench, 0x10, 1);
}

fn read_0x100_x1(bench: &mut Bencher) {
    read_test(bench, 0x100, 1);
}

fn read_0x1000_x1(bench: &mut Bencher) {
    read_test(bench, 0x1000, 1);
}

fn read_0x10000_x1(bench: &mut Bencher) {
    read_test(bench, 0x10000, 1);
}

benchmark_group!(
    benches,
    read_0x8_x1,
    read_0x10_x1,
    read_0x100_x1,
    read_0x1000_x1,
    read_0x10000_x1
);
benchmark_main!(benches);
