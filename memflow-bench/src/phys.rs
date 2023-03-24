use criterion::*;

use memflow::mem::{CachedPhysicalMemory, MemOps, PhysicalMemory};

use memflow::architecture;
use memflow::cglue::*;
use memflow::error::Result;
use memflow::mem::PhysicalReadData;
use memflow::types::*;

use rand::prelude::*;
use rand::{Rng, SeedableRng};
use rand_xorshift::XorShiftRng as CurRng;

use std::convert::TryInto;

fn rwtest(
    bench: &mut Bencher,
    mut mem: impl PhysicalMemory,
    (start, end): (Address, Address),
    chunk_sizes: &[usize],
    chunk_counts: &[usize],
    read_size: usize,
) -> usize {
    let mut rng = CurRng::from_rng(thread_rng()).unwrap();

    let mut total_size = 0;

    for i in chunk_sizes {
        for o in chunk_counts {
            let mut vbufs = vec![vec![0_u8; *i]; *o];
            let mut done_size = 0;

            while done_size < read_size {
                let base_addr = rng.gen_range(start.to_umem()..end.to_umem());

                let mut bufs = Vec::with_capacity(*o);

                bufs.extend(vbufs.iter_mut().map(|vec| {
                    let addr = (base_addr + rng.gen_range(0..0x2000)).into();

                    CTup3(
                        PhysicalAddress::with_page(
                            addr,
                            PageType::default().write(true),
                            mem::kb(4),
                        ),
                        Address::NULL,
                        vec.as_mut_slice().into(),
                    )
                }));

                bench.iter(|| {
                    let iter = bufs
                        .iter_mut()
                        .map(|CTup3(a, b, d): &mut PhysicalReadData| CTup3(*a, *b, d.into()));
                    let _ = black_box(MemOps::with_raw(iter, None, None, |data| {
                        mem.phys_read_raw_iter(data)
                    }));
                });

                done_size += *i * *o;
            }

            total_size += done_size
        }
    }

    total_size
}

fn read_test_with_mem(
    bench: &mut Bencher,
    mem: impl PhysicalMemory,
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

fn read_test_with_ctx(
    bench: &mut Bencher,
    cache_size: u64,
    chunk_size: usize,
    chunks: usize,
    mem: impl PhysicalMemory,
) {
    let mut rng = CurRng::from_rng(thread_rng()).unwrap();

    let start = Address::from(rng.gen_range(0..size::mb(50)));
    let end = start + size::mb(1);

    if cache_size > 0 {
        let mut cached_mem = CachedPhysicalMemory::builder(mem)
            .arch(architecture::x86::x64::ARCH)
            .cache_size(size::mb(cache_size as usize))
            .page_type_mask(PageType::PAGE_TABLE | PageType::READ_ONLY | PageType::WRITEABLE)
            .build()
            .unwrap();

        read_test_with_mem(
            bench,
            cached_mem.forward_mut(),
            chunk_size,
            chunks,
            (start, end),
        );
    } else {
        read_test_with_mem(bench, mem, chunk_size, chunks, (start, end));
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
                    black_box(size.try_into().unwrap()),
                    black_box(1),
                    initialize_ctx().unwrap().forward_mut(),
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
                BenchmarkId::new(format!("{func_name}_s{size:x}"), size * chunk_size),
                &size,
                |b, &size| {
                    read_test_with_ctx(
                        b,
                        black_box(cache_size),
                        black_box(size.try_into().unwrap()),
                        black_box(chunk_size.try_into().unwrap()),
                        initialize_ctx().unwrap().forward_mut(),
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

    let group_name = format!("{backend_name}_phys_seq_read");

    let mut group = c.benchmark_group(group_name.clone());
    group.plot_config(plot_config);

    seq_read_params(
        &mut group,
        format!("{group_name}_nocache"),
        0,
        initialize_ctx,
    );
    seq_read_params(&mut group, format!("{group_name}_cache"), 2, initialize_ctx);
}

pub fn chunk_read<T: PhysicalMemory>(
    c: &mut Criterion,
    backend_name: &str,
    initialize_ctx: &dyn Fn() -> Result<T>,
) {
    let plot_config = PlotConfiguration::default().summary_scale(AxisScale::Logarithmic);

    let group_name = format!("{backend_name}_phys_chunk_read");

    let mut group = c.benchmark_group(group_name.clone());
    group.plot_config(plot_config);

    chunk_read_params(
        &mut group,
        format!("{group_name}_nocache"),
        0,
        initialize_ctx,
    );
    chunk_read_params(&mut group, format!("{group_name}_cache"), 2, initialize_ctx);
}
