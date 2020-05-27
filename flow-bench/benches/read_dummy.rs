use std::time::Duration;

use criterion::*;

use flow_core::mem::{
    timed_validator::*, AccessVirtualMemory, CachedMemoryAccess, CachedVAT, PageCache, TLBCache,
};
use flow_core::{Address, Length, OsProcess, OsProcessModule, PageType};

use flow_core::mem::dummy::{DummyMemory as Memory, DummyModule, DummyProcess};

use rand::prelude::*;
use rand::{prng::XorShiftRng as CurRng, Rng, SeedableRng};

fn rwtest<T: AccessVirtualMemory>(
    mem: &mut T,
    proc: &DummyProcess,
    module: &dyn OsProcessModule,
    chunk_sizes: &[usize],
    chunk_counts: &[usize],
    read_size: usize,
) -> usize {
    let mut rng = CurRng::from_rng(thread_rng()).unwrap();

    let mut total_size = 0;

    for i in chunk_sizes {
        for o in chunk_counts {
            let mut bufs = vec![(Address::null(), vec![0 as u8; *i]); *o];
            let mut done_size = 0;

            while done_size < read_size {
                let base_addr = rng.gen_range(
                    module.base().as_u64(),
                    module.base().as_u64() + module.size().as_u64(),
                );
                for (addr, _) in bufs.iter_mut() {
                    *addr = (base_addr + rng.gen_range(0, 0x2000)).into();
                }

                {
                    let mut vmem = proc.virt_mem(mem);
                    for (addr, buf) in bufs.iter_mut() {
                        let _ = vmem.virt_read_raw_into(*addr, buf.as_mut_slice());
                    }
                }
                done_size += *i * *o;
            }

            total_size += done_size
        }
    }

    total_size
}

fn initialize_ctx() -> flow_core::Result<(Memory, DummyProcess, DummyModule)> {
    let mut mem = Memory::new(Length::from_mb(64));

    let proc = mem.alloc_process(Length::from_mb(60), &[]);
    let module = proc.get_module(Length::from_mb(4));

    Ok((mem, proc, module))
}

fn read_test_with_mem<T: AccessVirtualMemory>(
    bench: &mut Bencher,
    mem: &mut T,
    chunk_size: usize,
    chunks: usize,
    proc: DummyProcess,
    tmod: DummyModule,
) {
    bench.iter(|| {
        black_box(rwtest(
            mem,
            &proc,
            &tmod,
            &[chunk_size],
            &[chunks],
            chunk_size,
        ));
    });
}

fn read_test(
    bench: &mut Bencher,
    cache_size: u64,
    chunk_size: usize,
    chunks: usize,
    use_tlb: bool,
) {
    let (mut mem, proc, tmod) = initialize_ctx().unwrap();

    let tlb_cache = TLBCache::new(
        2048.into(),
        TimedCacheValidator::new(Duration::from_millis(1000).into()),
    );

    if cache_size > 0 {
        let cache = PageCache::new(
            proc.sys_arch(),
            Length::from_mb(cache_size),
            PageType::PAGE_TABLE | PageType::READ_ONLY | PageType::WRITEABLE,
            TimedCacheValidator::new(Duration::from_millis(10000).into()),
        );

        if use_tlb {
            let mem = CachedMemoryAccess::with(&mut mem, cache);
            let mut mem = CachedVAT::with(mem, tlb_cache);
            read_test_with_mem(bench, &mut mem, chunk_size, chunks, proc, tmod);
        } else {
            let mut mem = CachedMemoryAccess::with(&mut mem, cache);
            read_test_with_mem(bench, &mut mem, chunk_size, chunks, proc, tmod);
        }
    } else if use_tlb {
        let mut mem = CachedVAT::with(mem, tlb_cache);
        read_test_with_mem(bench, &mut mem, chunk_size, chunks, proc, tmod);
    } else {
        read_test_with_mem(bench, &mut mem, chunk_size, chunks, proc, tmod);
    }
}

fn dummy_read_params(
    group: &mut BenchmarkGroup<'_, measurement::WallTime>,
    func_name: String,
    cache_size: u64,
    use_tlb: bool,
) {
    for &chunk_count in [1, 2].iter() {
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
fn dummy_read_group(c: &mut Criterion) {
    let plot_config = PlotConfiguration::default().summary_scale(AxisScale::Logarithmic);

    let mut group = c.benchmark_group("dummy_read");
    group.plot_config(plot_config);

    dummy_read_params(&mut group, "dummy_read_nocache".into(), 0, false);
    dummy_read_params(&mut group, "dummy_read_tlb_nocache".into(), 0, true);
    dummy_read_params(&mut group, "dummy_read_cache".into(), 2, false);
    dummy_read_params(&mut group, "dummy_read_tlb_cache".into(), 2, true);
}

criterion_group! {
    name = dummy_read;
    config = Criterion::default()
        .warm_up_time(std::time::Duration::from_millis(500))
        .measurement_time(std::time::Duration::from_secs(2));
    targets = dummy_read_group
}

criterion_main!(dummy_read);
