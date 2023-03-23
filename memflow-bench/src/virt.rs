use criterion::*;

use memflow::mem::MemoryView;

use memflow::cglue::*;
use memflow::error::Result;
use memflow::os::*;
use memflow::plugins::*;

use rand::prelude::*;
use rand::{Rng, SeedableRng};
use rand_xorshift::XorShiftRng as CurRng;

use std::convert::TryInto;

fn rwtest<T: MemoryView>(
    bench: &mut Bencher,
    virt_mem: &mut T,
    module: &ModuleInfo,
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
                let base_addr =
                    rng.gen_range(module.base.to_umem()..(module.base.to_umem() + module.size));

                let mut bufs = Vec::with_capacity(*o);

                for CTup2(addr, _) in bufs.iter_mut() {
                    *addr = (base_addr + rng.gen_range(0..0x2000)).into();
                }

                bufs.extend(vbufs.iter_mut().map(|vec| {
                    CTup2(
                        (base_addr + rng.gen_range(0..0x2000)).into(),
                        vec.as_mut_slice().into(),
                    )
                }));

                bench.iter(|| {
                    let _ = black_box(virt_mem.read_raw_list(bufs.as_mut_slice()));
                });
                done_size += *i * *o;
            }

            total_size += done_size
        }
    }

    total_size
}

pub fn read_test_with_mem<T: MemoryView>(
    bench: &mut Bencher,
    virt_mem: &mut T,
    chunk_size: usize,
    chunks: usize,
    tmod: ModuleInfo,
) {
    black_box(rwtest(
        bench,
        virt_mem,
        &tmod,
        &[chunk_size],
        &[chunks],
        chunk_size,
    ));
}

fn read_test_with_os(
    bench: &mut Bencher,
    chunk_size: usize,
    chunks: usize,
    os: &mut OsInstanceArcBox<'static>,
) {
    let (mut proc, module) = crate::util::find_proc(os).unwrap();
    read_test_with_mem(bench, &mut proc, chunk_size, chunks, module);
}

fn seq_read_params(
    group: &mut BenchmarkGroup<'_, measurement::WallTime>,
    func_name: String,
    cache_size: usize,
    use_tlb: bool,
    initialize_ctx: &dyn Fn(usize, bool) -> Result<OsInstanceArcBox<'static>>,
) {
    let mut os = initialize_ctx(cache_size, use_tlb).unwrap();

    for &size in [0x8, 0x10, 0x100, 0x1000, 0x10000].iter() {
        group.throughput(Throughput::Bytes(size));
        group.bench_with_input(
            BenchmarkId::new(func_name.clone(), size),
            &size,
            |b, &size| {
                read_test_with_os(
                    b,
                    black_box(size.try_into().unwrap()),
                    black_box(1),
                    &mut os,
                )
            },
        );
    }
}

fn chunk_read_params(
    group: &mut BenchmarkGroup<'_, measurement::WallTime>,
    func_name: String,
    cache_size: usize,
    use_tlb: bool,
    initialize_ctx: &dyn Fn(usize, bool) -> Result<OsInstanceArcBox<'static>>,
) {
    let mut os = initialize_ctx(cache_size, use_tlb).unwrap();

    for &size in [0x8, 0x10, 0x100, 0x1000].iter() {
        for &chunk_size in [1, 4, 16, 64].iter() {
            group.throughput(Throughput::Bytes(size * chunk_size));
            group.bench_with_input(
                BenchmarkId::new(format!("{func_name}_s{size:x}"), size * chunk_size),
                &size,
                |b, &size| {
                    read_test_with_os(
                        b,
                        black_box(size.try_into().unwrap()),
                        black_box(chunk_size.try_into().unwrap()),
                        &mut os,
                    )
                },
            );
        }
    }
}

pub fn seq_read(
    c: &mut Criterion,
    backend_name: &str,
    initialize_ctx: &dyn Fn(usize, bool) -> Result<OsInstanceArcBox<'static>>,
    use_caches: bool,
) {
    let plot_config = PlotConfiguration::default().summary_scale(AxisScale::Logarithmic);

    let group_name = format!("{backend_name}_virt_seq_read");

    let mut group = c.benchmark_group(group_name.clone());
    group.plot_config(plot_config);

    seq_read_params(
        &mut group,
        format!("{group_name}_nocache"),
        0,
        false,
        initialize_ctx,
    );
    if use_caches {
        seq_read_params(
            &mut group,
            format!("{group_name}_tlb_nocache"),
            0,
            true,
            initialize_ctx,
        );
        seq_read_params(
            &mut group,
            format!("{group_name}_cache"),
            2,
            false,
            initialize_ctx,
        );
        seq_read_params(
            &mut group,
            format!("{group_name}_tlb_cache"),
            2,
            true,
            initialize_ctx,
        );
    }
}

pub fn chunk_read(
    c: &mut Criterion,
    backend_name: &str,
    initialize_ctx: &dyn Fn(usize, bool) -> Result<OsInstanceArcBox<'static>>,
    use_caches: bool,
) {
    let plot_config = PlotConfiguration::default().summary_scale(AxisScale::Logarithmic);

    let group_name = format!("{backend_name}_virt_chunk_read");

    let mut group = c.benchmark_group(group_name.clone());
    group.plot_config(plot_config);

    chunk_read_params(
        &mut group,
        format!("{group_name}_nocache"),
        0,
        false,
        initialize_ctx,
    );

    if use_caches {
        chunk_read_params(
            &mut group,
            format!("{group_name}_tlb_nocache"),
            0,
            true,
            initialize_ctx,
        );
        chunk_read_params(
            &mut group,
            format!("{group_name}_cache"),
            2,
            false,
            initialize_ctx,
        );
        chunk_read_params(
            &mut group,
            format!("{group_name}_tlb_cache"),
            2,
            true,
            initialize_ctx,
        );
    }
}
