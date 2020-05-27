
extern crate flow_bench;
use flow_bench::*;

use criterion::*;

use flow_core::OsProcessModule;

use flow_qemu_procfs::Memory;
use flow_win32::{Win32, Win32Module, Win32Offsets, Win32Process};

use rand::prelude::*;
use rand::{prng::XorShiftRng as CurRng, Rng, SeedableRng};

fn initialize_ctx() -> flow_core::Result<(Memory, Win32Process, Win32Module)> {
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
                return Ok((mem, proc, tmod.clone()));
            }
        }
    }

    Err("No module found!".into())
}

fn read_test(
    bench: &mut Bencher,
    cache_size: u64,
    chunk_size: usize,
    chunks: usize,
    use_tlb: bool,
) {
    let (mem, proc, tmod) = initialize_ctx().unwrap();
    read_test_with_ctx(bench, cache_size, chunk_size, chunks, use_tlb, mem, proc, tmod);
}

fn win32_read_params(
    group: &mut BenchmarkGroup<'_, measurement::WallTime>,
    func_name: String,
    cache_size: u64,
    use_tlb: bool,
    max_chunk_pow: usize
) {
    for i in 0..(max_chunk_pow+1) {
        let chunk_count = 1 << i;
        for &size in [0x8, 0x10, 0x100, 0x1000, 0x10000].iter() {
            group.throughput(Throughput::Bytes(size * chunk_count));
            group.bench_with_input(
                BenchmarkId::new(format!("{}_{}_chunks", func_name, chunk_count), size),
                &(size, chunk_count),
                |b, &(size, chunk_count)| {
                    read_test(
                        b,
                        black_box(cache_size),
                        black_box(size as usize),
                        black_box(chunk_count as usize),
                        black_box(use_tlb),
                    )
                },
            );
        }
    }
}

fn win32_read_group(c: &mut Criterion) {
    let plot_config = PlotConfiguration::default().summary_scale(AxisScale::Logarithmic);

    let mut group = c.benchmark_group("win32_read");
    group.plot_config(plot_config);

    win32_read_params(&mut group, "win32_read_nocache".into(), 0, false, 0);
    win32_read_params(&mut group, "win32_read_tlb_nocache".into(), 0, true, 0);
    win32_read_params(&mut group, "win32_read_cache".into(), 2, false, 0);
    win32_read_params(&mut group, "win32_read_tlb_cache".into(), 2, true, 0);
}

criterion_group! {
    name = win32_read;
    config = Criterion::default()
        .warm_up_time(std::time::Duration::from_millis(500))
        .measurement_time(std::time::Duration::from_secs(2));
    targets = win32_read_group
}

criterion_main!(win32_read);
