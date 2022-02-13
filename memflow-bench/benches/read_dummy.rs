extern crate memflow_bench;
use memflow_bench::*;

use criterion::*;

use memflow::dummy::DummyMemory as Memory;
use memflow::prelude::v1::*;

fn initialize_virt_ctx(cache_size: usize, use_tlb: bool) -> Result<OsInstanceArcBox<'static>> {
    util::build_os("", cache_size, "dummy", use_tlb)
}

fn dummy_read_group(c: &mut Criterion) {
    virt::seq_read(c, "dummy", &initialize_virt_ctx, false);
    virt::chunk_read(c, "dummy", &initialize_virt_ctx, false);
    phys::seq_read(c, "dummy", &|| Ok(Memory::new(size::mb(64))));
    phys::chunk_read(c, "dummy", &|| Ok(Memory::new(size::mb(64))));
    vat::chunk_vat(c, "dummy", &initialize_virt_ctx, false);
}

criterion_group! {
    name = dummy_read;
    config = Criterion::default()
        .warm_up_time(std::time::Duration::from_millis(1000))
        .measurement_time(std::time::Duration::from_millis(10000));
    targets = dummy_read_group
}

criterion_main!(dummy_read);
