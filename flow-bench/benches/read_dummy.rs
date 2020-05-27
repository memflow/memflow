
extern crate flow_bench;
use flow_bench::*;

use criterion::*;

use flow_core::Length;

use flow_core::mem::dummy::{DummyMemory as Memory, DummyModule, DummyProcess};

fn initialize_ctx() -> flow_core::Result<(Memory, DummyProcess, DummyModule)> {
    let mut mem = Memory::new(Length::from_mb(64));

    let proc = mem.alloc_process(Length::from_mb(60), &[]);
    let module = proc.get_module(Length::from_mb(4));

    Ok((mem, proc, module))
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

fn dummy_read_params(
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

fn dummy_read_group(c: &mut Criterion) {
    let plot_config = PlotConfiguration::default().summary_scale(AxisScale::Logarithmic);

    let mut group = c.benchmark_group("dummy_read");
    group.plot_config(plot_config);

    dummy_read_params(&mut group, "dummy_read_nocache".into(), 0, false, 0);
    dummy_read_params(&mut group, "dummy_read_tlb_nocache".into(), 0, true, 0);
    dummy_read_params(&mut group, "dummy_read_cache".into(), 2, false, 0);
    dummy_read_params(&mut group, "dummy_read_tlb_cache".into(), 2, true, 0);
}

criterion_group! {
    name = dummy_read;
    config = Criterion::default()
        .warm_up_time(std::time::Duration::from_millis(500))
        .measurement_time(std::time::Duration::from_secs(2));
    targets = dummy_read_group
}

criterion_main!(dummy_read);
