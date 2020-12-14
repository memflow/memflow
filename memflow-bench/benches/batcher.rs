use criterion::*;

use memflow::prelude::v1::*;

//use memflow::mem::dummy::DummyMemory as Memory;

struct NullMem {}

impl NullMem {
    pub fn new(_: usize) -> Self {
        Self {}
    }
}

impl PhysicalMemory for NullMem {
    fn phys_read_raw_list(&mut self, data: &mut [PhysicalReadData]) -> Result<()> {
        black_box(data.iter_mut().count());
        Ok(())
    }

    fn phys_write_raw_list(&mut self, data: &[PhysicalWriteData]) -> Result<()> {
        black_box(data.iter().count());
        Ok(())
    }

    fn metadata(&self) -> PhysicalMemoryMetadata {
        PhysicalMemoryMetadata {
            size: 0,
            readonly: true,
        }
    }
}

use NullMem as Memory;

use rand::prelude::*;
use rand::{Rng, SeedableRng};
use rand_xorshift::XorShiftRng as CurRng;

static mut TSLICE: [[u8; 16]; 0x10000] = [[0; 16]; 0x10000];

fn read_test_nobatcher<T: PhysicalMemory>(
    chunk_size: usize,
    mem: &mut T,
    mut rng: CurRng,
    size: usize,
    tbuf: &mut [PhysicalReadData],
) {
    let base_addr = Address::from(rng.gen_range(0, size));

    for PhysicalReadData(addr, _) in tbuf.iter_mut().take(chunk_size) {
        *addr = (base_addr + rng.gen_range(0, 0x2000)).into();
    }

    let _ = black_box(mem.phys_read_raw_list(&mut tbuf[..chunk_size]));
}

fn read_test_batcher<T: PhysicalMemory>(
    chunk_size: usize,
    mem: &mut T,
    mut rng: CurRng,
    size: usize,
) {
    let base_addr = Address::from(rng.gen_range(0, size));

    let mut batcher = mem.phys_batcher();
    batcher.read_prealloc(chunk_size);

    for i in unsafe { TSLICE.iter_mut().take(chunk_size) } {
        batcher.read_into((base_addr + rng.gen_range(0, 0x2000)).into(), i);
    }

    let _ = black_box(batcher.commit_rw());
}

fn read_test_with_ctx<T: PhysicalMemory>(
    bench: &mut Bencher,
    chunk_size: usize,
    use_batcher: bool,
    mem: &mut T,
) {
    let rng = CurRng::from_rng(thread_rng()).unwrap();

    let mem_size = size::mb(64);

    let mut tbuf = vec![];

    tbuf.extend(
        unsafe { TSLICE }
            .iter_mut()
            .map(|arr| {
                PhysicalReadData(PhysicalAddress::INVALID, unsafe {
                    std::mem::transmute(&mut arr[..])
                })
            })
            .take(chunk_size),
    );

    if !use_batcher {
        bench.iter(move || {
            read_test_nobatcher(chunk_size, mem, rng.clone(), mem_size, &mut tbuf[..])
        });
    } else {
        bench.iter(|| read_test_batcher(chunk_size, mem, rng.clone(), mem_size));
    }
}

fn chunk_read_params<T: PhysicalMemory>(
    group: &mut BenchmarkGroup<'_, measurement::WallTime>,
    func_name: String,
    use_batcher: bool,
    initialize_ctx: &dyn Fn() -> T,
) {
    for &chunk_size in [1, 4, 16, 64, 256, 1024, 4096, 16384, 65536].iter() {
        group.throughput(Throughput::Bytes(chunk_size));
        group.bench_with_input(
            BenchmarkId::new(func_name.clone(), chunk_size),
            &chunk_size,
            |b, &chunk_size| {
                read_test_with_ctx(
                    b,
                    black_box(chunk_size as usize),
                    use_batcher,
                    &mut initialize_ctx(),
                )
            },
        );
    }
}

fn chunk_read<T: PhysicalMemory>(
    c: &mut Criterion,
    backend_name: &str,
    initialize_ctx: &dyn Fn() -> T,
) {
    let plot_config = PlotConfiguration::default().summary_scale(AxisScale::Logarithmic);

    let group_name = format!("{}_batched_read", backend_name);

    let mut group = c.benchmark_group(group_name.clone());
    group.plot_config(plot_config);

    chunk_read_params(
        &mut group,
        format!("{}_without", group_name),
        false,
        initialize_ctx,
    );
    chunk_read_params(
        &mut group,
        format!("{}_with", group_name),
        true,
        initialize_ctx,
    );
}
criterion_group! {
    name = dummy_read;
    config = Criterion::default()
        .warm_up_time(std::time::Duration::from_millis(300))
        .measurement_time(std::time::Duration::from_millis(2700));
    targets = dummy_read_group
}

fn dummy_read_group(c: &mut Criterion) {
    chunk_read(c, "dummy", &|| Memory::new(size::mb(64)));
}

criterion_main!(dummy_read);
