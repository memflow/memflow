use criterion::*;

use memflow::prelude::v1::*;

use std::convert::TryInto;

//use memflow::dummy::DummyMemory as Memory;

struct NullMem {}

impl NullMem {
    pub fn new(_: umem) -> Self {
        Self {}
    }
}

impl PhysicalMemory for NullMem {
    fn phys_read_raw_iter(&mut self, data: PhysicalReadMemOps) -> Result<()> {
        black_box(data);
        Ok(())
    }

    fn phys_write_raw_iter(&mut self, data: PhysicalWriteMemOps) -> Result<()> {
        black_box(data);
        Ok(())
    }

    fn metadata(&self) -> PhysicalMemoryMetadata {
        PhysicalMemoryMetadata {
            max_address: Address::NULL,
            real_size: 0,
            readonly: true,
            ideal_batch_size: u32::MAX,
        }
    }
}

use NullMem as Memory;

use rand::prelude::*;
use rand::{Rng, SeedableRng};
use rand_xorshift::XorShiftRng as CurRng;

static mut TSLICE: [[u8; 16]; 0x10000] = [[0; 16]; 0x10000];

fn read_test_nobatcher<T: MemoryView>(
    chunk_size: usize,
    mem: &mut T,
    mut rng: CurRng,
    size: umem,
    tbuf: &mut [ReadDataRaw],
) {
    let base_addr = Address::from(rng.gen_range(0..size));

    for CTup3(addr, _, _) in tbuf.iter_mut().take(chunk_size) {
        *addr = base_addr + rng.gen_range(0usize..0x2000);
    }

    let iter = tbuf[..chunk_size]
        .iter_mut()
        .map(|CTup3(a, b, c): &mut ReadDataRaw| CTup3(*a, *b, c.into()));

    let _ = black_box(MemOps::with_raw(iter, None, None, |data| {
        mem.read_raw_iter(data)
    }));
}

fn read_test_batcher<T: MemoryView>(chunk_size: usize, mem: &mut T, mut rng: CurRng, size: umem) {
    let base_addr = Address::from(rng.gen_range(0..size));

    let mut batcher = mem.batcher();
    batcher.reserve(chunk_size);

    for i in unsafe { TSLICE.iter_mut().take(chunk_size) } {
        batcher.read_into(base_addr + rng.gen_range(0usize..0x2000), i);
    }

    let _ = black_box(batcher.commit_rw());
}

fn read_test_with_ctx<T: MemoryView>(
    bench: &mut Bencher,
    chunk_size: usize,
    use_batcher: bool,
    mem: &mut T,
) {
    let rng = CurRng::from_rng(thread_rng()).unwrap();

    let mem_size = mem::mb(64);

    let mut tbuf = vec![];

    tbuf.extend(
        unsafe { TSLICE }
            .iter_mut()
            .map(|arr| {
                CTup3(Address::INVALID, Address::INVALID, unsafe {
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
                    black_box(chunk_size.try_into().unwrap()),
                    use_batcher,
                    &mut initialize_ctx().into_phys_view(),
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

    let group_name = format!("{backend_name}_batched_read");

    let mut group = c.benchmark_group(group_name.clone());
    group.plot_config(plot_config);

    chunk_read_params(
        &mut group,
        format!("{group_name}_without"),
        false,
        initialize_ctx,
    );
    chunk_read_params(
        &mut group,
        format!("{group_name}_with"),
        true,
        initialize_ctx,
    );
}
criterion_group! {
    name = dummy_read;
    config = Criterion::default()
        .warm_up_time(std::time::Duration::from_millis(500))
        .measurement_time(std::time::Duration::from_millis(5000));
    targets = dummy_read_group
}

fn dummy_read_group(c: &mut Criterion) {
    chunk_read(c, "dummy", &|| Memory::new(mem::mb(64)));
}

criterion_main!(dummy_read);
