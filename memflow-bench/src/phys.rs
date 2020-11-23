use criterion::*;

use memflow::mem::{CachedMemoryAccess, PhysicalMemory};

use memflow::architecture;
use memflow::error::Result;
use memflow::mem::PhysicalReadData;
use memflow::types::*;

use rand::prelude::*;
use rand::{Rng, SeedableRng};
use rand_xorshift::XorShiftRng as CurRng;

fn rwtest<T: PhysicalMemory>(
    bench: &mut Bencher,
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
            let mut vbufs = vec![vec![0 as u8; *i]; *o];
            let mut done_size = 0;

            while done_size < read_size {
                let base_addr = rng.gen_range(start.as_u64(), end.as_u64());

                let mut bufs = Vec::with_capacity(*o);

                bufs.extend(vbufs.iter_mut().map(|vec| {
                    let addr = (base_addr + rng.gen_range(0, 0x2000)).into();

                    PhysicalReadData(
                        PhysicalAddress::with_page(
                            addr,
                            PageType::default().write(true),
                            size::kb(4),
                        ),
                        vec.as_mut_slice(),
                    )
                }));

                bench.iter(|| {
                    let _ = black_box(mem.phys_read_raw_list(&mut bufs));
                });

                done_size += *i * *o;
            }

            total_size += done_size
        }
    }

    total_size
}

fn read_test_with_mem<T: PhysicalMemory>(
    bench: &mut Bencher,
    mem: &mut T,
    chunk_size: usize,
    chunks: usize,
    start_end: (Address, Address),
) {
    black_box(rwtest(
        bench,
        mem,
        start_end,
        &[chunk_size],
        &[chunks],
        chunk_size,
    ));
}

fn read_test_with_ctx<T: PhysicalMemory>(
    bench: &mut Bencher,
    cache_size: u64,
    chunk_size: usize,
    chunks: usize,
    mut mem: T,
) {
    let mut rng = CurRng::from_rng(thread_rng()).unwrap();

    let start = Address::from(rng.gen_range(0, size::mb(50)));
    let end = start + size::mb(1);

    if cache_size > 0 {
        let mut mem = CachedMemoryAccess::builder(&mut mem)
            .arch(architecture::x86::x64::ARCH)
            .cache_size(size::mb(cache_size as usize))
            .page_type_mask(PageType::PAGE_TABLE | PageType::READ_ONLY | PageType::WRITEABLE)
            .build()
            .unwrap();

        read_test_with_mem(bench, &mut mem, chunk_size, chunks, (start, end));
    } else {
        read_test_with_mem(bench, &mut mem, chunk_size, chunks, (start, end));
    }
}

fn seq_read_params<T: PhysicalMemory>(
    group: &mut BenchmarkGroup<'_, measurement::WallTime>,
    func_name: String,
    cache_size: u64,
    initialize_ctx: &dyn Fn() -> Result<T>,
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

fn chunk_read_params<T: PhysicalMemory>(
    group: &mut BenchmarkGroup<'_, measurement::WallTime>,
    func_name: String,
    cache_size: u64,
    initialize_ctx: &dyn Fn() -> Result<T>,
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

pub fn seq_read<T: PhysicalMemory>(
    c: &mut Criterion,
    backend_name: &str,
    initialize_ctx: &dyn Fn() -> Result<T>,
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

pub fn chunk_read<T: PhysicalMemory>(
    c: &mut Criterion,
    backend_name: &str,
    initialize_ctx: &dyn Fn() -> Result<T>,
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
