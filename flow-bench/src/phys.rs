use criterion::*;

use flow_core::mem::{timed_validator::*, AccessPhysicalMemory, CachedMemoryAccess, PageCache};

use flow_core::{Address, Architecture, Length, PageType, PhysicalAddress};

use rand::prelude::*;
use rand::{prng::XorShiftRng as CurRng, Rng, SeedableRng};

fn rwtest<T: AccessPhysicalMemory>(
    mem: &mut T,
    (start, end): (Address, Address),
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
                let base_addr = rng.gen_range(start.as_u64(), end.as_u64());

                let _ = mem.phys_read_iter(bufs.iter_mut().map(|(addr, buf)| {
                    *addr = (base_addr + rng.gen_range(0, 0x2000)).into();
                    (
                        PhysicalAddress::with_page(
                            *addr,
                            PageType::from_writeable_bit(true),
                            Length::from_kb(4),
                        ),
                        buf.as_mut_slice(),
                    )
                }));
                done_size += *i * *o;
            }

            total_size += done_size
        }
    }

    total_size
}

pub fn read_test_with_mem<T: AccessPhysicalMemory>(
    bench: &mut Bencher,
    mem: &mut T,
    chunk_size: usize,
    chunks: usize,
    start_end: (Address, Address),
) {
    bench.iter(|| {
        black_box(rwtest(mem, start_end, &[chunk_size], &[chunks], chunk_size));
    });
}

fn read_test_with_ctx<T: AccessPhysicalMemory>(
    bench: &mut Bencher,
    cache_size: u64,
    chunk_size: usize,
    chunks: usize,
    mut mem: T,
) {
    let mut rng = CurRng::from_rng(thread_rng()).unwrap();

    let start = Address::from(rng.gen_range(0, Length::from_mb(50).as_u64()));
    let end = start + Length::from_mb(1);

    if cache_size > 0 {
        let cache = PageCache::new(
            Architecture::X64,
            Length::from_mb(cache_size),
            PageType::PAGE_TABLE | PageType::READ_ONLY | PageType::WRITEABLE,
            TimedCacheValidator::new(Duration::from_millis(10000).into()),
        );

        let mut mem = CachedMemoryAccess::with(&mut mem, cache);
        read_test_with_mem(bench, &mut mem, chunk_size, chunks, (start, end));
    } else {
        read_test_with_mem(bench, &mut mem, chunk_size, chunks, (start, end));
    }
}

fn seq_read_params<T: AccessPhysicalMemory>(
    group: &mut BenchmarkGroup<'_, measurement::WallTime>,
    func_name: String,
    cache_size: u64,
    initialize_ctx: &dyn Fn() -> flow_core::Result<T>,
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
                    initialize_ctx().unwrap(),
                )
            },
        );
    }
}

fn chunk_read_params<T: AccessPhysicalMemory>(
    group: &mut BenchmarkGroup<'_, measurement::WallTime>,
    func_name: String,
    cache_size: u64,
    initialize_ctx: &dyn Fn() -> flow_core::Result<T>,
) {
    for &size in [0x8, 0x10, 0x100, 0x1000].iter() {
        for &chunk_size in [1, 4, 16, 64].iter() {
            group.throughput(Throughput::Bytes(size * chunk_size));
            group.bench_with_input(
                BenchmarkId::new(format!("{}_s{:x}", func_name, size), size * chunk_size),
                &size,
                |b, &size| {
                    read_test_with_ctx(
                        b,
                        black_box(cache_size),
                        black_box(size as usize),
                        black_box(chunk_size as usize),
                        initialize_ctx().unwrap(),
                    )
                },
            );
        }
    }
}

pub fn seq_read<T: AccessPhysicalMemory>(
    c: &mut Criterion,
    backend_name: &str,
    initialize_ctx: &dyn Fn() -> flow_core::Result<T>,
) {
    let plot_config = PlotConfiguration::default().summary_scale(AxisScale::Logarithmic);

    let group_name = format!("{}_phys_seq_read", backend_name);

    let mut group = c.benchmark_group(group_name.clone());
    group.plot_config(plot_config);

    seq_read_params(
        &mut group,
        format!("{}_nocache", group_name),
        0,
        initialize_ctx,
    );
    seq_read_params(
        &mut group,
        format!("{}_cache", group_name),
        2,
        initialize_ctx,
    );
}

pub fn chunk_read<T: AccessPhysicalMemory>(
    c: &mut Criterion,
    backend_name: &str,
    initialize_ctx: &dyn Fn() -> flow_core::Result<T>,
) {
    let plot_config = PlotConfiguration::default().summary_scale(AxisScale::Logarithmic);

    let group_name = format!("{}_phys_chunk_read", backend_name);

    let mut group = c.benchmark_group(group_name.clone());
    group.plot_config(plot_config);

    chunk_read_params(
        &mut group,
        format!("{}_nocache", group_name),
        0,
        initialize_ctx,
    );
    chunk_read_params(
        &mut group,
        format!("{}_cache", group_name),
        2,
        initialize_ctx,
    );
}
