use criterion::*;

use flow_core::mem::{
    timed_validator::*, vat::VirtualAddressTranslator, AccessPhysicalMemory, AccessVirtualMemory,
    CachedMemoryAccess, CachedVAT, PageCache, TLBCache,
};

use flow_core::{Address, Length, OsProcess, OsProcessModule, PageType};

use rand::prelude::*;
use rand::{prng::XorShiftRng as CurRng, Rng, SeedableRng};

fn rwtest<T: AccessVirtualMemory, P: OsProcess, M: OsProcessModule>(
    mem: &mut T,
    proc: &P,
    module: &M,
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

pub fn read_test_with_mem<T: AccessVirtualMemory, P: OsProcess, M: OsProcessModule>(
    bench: &mut Bencher,
    mem: &mut T,
    chunk_size: usize,
    chunks: usize,
    proc: P,
    tmod: M,
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

fn read_test_with_ctx<
    T: VirtualAddressTranslator + AccessVirtualMemory + AccessPhysicalMemory,
    P: OsProcess,
    M: OsProcessModule,
>(
    bench: &mut Bencher,
    cache_size: u64,
    chunk_size: usize,
    chunks: usize,
    use_tlb: bool,
    (mut mem, proc, tmod): (T, P, M),
) {
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

fn seq_read_params<
    T: VirtualAddressTranslator + AccessVirtualMemory + AccessPhysicalMemory,
    P: OsProcess,
    M: OsProcessModule,
>(
    group: &mut BenchmarkGroup<'_, measurement::WallTime>,
    func_name: String,
    cache_size: u64,
    use_tlb: bool,
    initialize_ctx: &dyn Fn() -> flow_core::Result<(T, P, M)>,
) {
    for &size in [0x8, 0x10, 0x100, 0x1000, 0x10000].iter() {
        group.throughput(Throughput::Bytes(size));
        group.bench_with_input(
            BenchmarkId::new(func_name.clone(), size),
            &size,
            |b, &size| {
                read_test_with_ctx(
                    b,
                    black_box(cache_size),
                    black_box(size as usize),
                    black_box(1),
                    black_box(use_tlb),
                    initialize_ctx().unwrap(),
                )
            },
        );
    }
}

fn chunk_read_params<
    T: VirtualAddressTranslator + AccessVirtualMemory + AccessPhysicalMemory,
    P: OsProcess,
    M: OsProcessModule,
>(
    group: &mut BenchmarkGroup<'_, measurement::WallTime>,
    func_name: String,
    cache_size: u64,
    use_tlb: bool,
    initialize_ctx: &dyn Fn() -> flow_core::Result<(T, P, M)>,
) {
    for &size in [0x8, 0x10, 0x100, 0x1000].iter() {
        for &chunk_size in [1, 4, 16, 64].iter() {
            group.throughput(Throughput::Bytes(size * chunk_size));
            group.bench_with_input(
                BenchmarkId::new(format!("{}_s{:x}", func_name, size), size),
                &size,
                |b, &size| {
                    read_test_with_ctx(
                        b,
                        black_box(cache_size),
                        black_box(size as usize),
                        black_box(chunk_size as usize),
                        black_box(use_tlb),
                        initialize_ctx().unwrap(),
                    )
                },
            );
        }
    }
}

pub fn seq_read<
    T: VirtualAddressTranslator + AccessVirtualMemory + AccessPhysicalMemory,
    P: OsProcess,
    M: OsProcessModule,
>(
    c: &mut Criterion,
    backend_name: &str,
    initialize_ctx: &dyn Fn() -> flow_core::Result<(T, P, M)>,
) {
    let plot_config = PlotConfiguration::default().summary_scale(AxisScale::Logarithmic);

    let group_name = format!("{}_virt_seq_read", backend_name);

    let mut group = c.benchmark_group(group_name.clone());
    group.plot_config(plot_config);

    seq_read_params(
        &mut group,
        format!("{}_nocache", group_name),
        0,
        false,
        initialize_ctx,
    );
    //seq_read_params(&mut group, format!("{}_tlb_nocache", group_name), 0, true, initialize_ctx);
    seq_read_params(
        &mut group,
        format!("{}_cache", group_name),
        2,
        false,
        initialize_ctx,
    );
    //seq_read_params(&mut group, format!("{}_tlb_cache", group_name), 2, true, initialize_ctx);
}

pub fn chunk_read<
    T: VirtualAddressTranslator + AccessVirtualMemory + AccessPhysicalMemory,
    P: OsProcess,
    M: OsProcessModule,
>(
    c: &mut Criterion,
    backend_name: &str,
    initialize_ctx: &dyn Fn() -> flow_core::Result<(T, P, M)>,
) {
    let plot_config = PlotConfiguration::default().summary_scale(AxisScale::Logarithmic);

    let group_name = format!("{}_virt_chunk_read", backend_name);

    let mut group = c.benchmark_group(group_name.clone());
    group.plot_config(plot_config);

    chunk_read_params(
        &mut group,
        format!("{}_nocache", group_name),
        0,
        false,
        initialize_ctx,
    );
    //chunk_read_params(&mut group, format!("{}_tlb_nocache", group_name), 0, true, initialize_ctx);
    chunk_read_params(
        &mut group,
        format!("{}_cache", group_name),
        2,
        false,
        initialize_ctx,
    );
    //chunk_read_params(&mut group, format!("{}_tlb_cache", group_name), 2, true, initialize_ctx);
}
