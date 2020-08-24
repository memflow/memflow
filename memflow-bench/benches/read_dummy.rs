extern crate memflow_bench;
use memflow_bench::*;

use criterion::*;

use memflow_core::architecture::{Architecture, ScopedVirtualTranslate};
use memflow_core::mem::dummy::{DummyMemory as Memory, DummyModule, DummyProcess};
use memflow_core::mem::DirectTranslate;
use memflow_core::types::size;

fn initialize_virt_ctx() -> memflow_core::Result<(
    Memory,
    DirectTranslate,
    DummyProcess,
    impl ScopedVirtualTranslate,
    DummyModule,
)> {
    let mut mem = Memory::new(size::mb(64));

    let vat = DirectTranslate::new();

    let proc = mem.alloc_process(size::mb(60), &[]);
    let module = proc.get_module(size::mb(4));
    let translator = proc.translator();
    Ok((mem, vat, proc, translator, module))
}

fn dummy_read_group(c: &mut Criterion) {
    virt::seq_read(c, "dummy", &initialize_virt_ctx);
    virt::chunk_read(c, "dummy", &initialize_virt_ctx);
    phys::seq_read(c, "dummy", &|| Ok(Memory::new(size::mb(64))));
    phys::chunk_read(c, "dummy", &|| Ok(Memory::new(size::mb(64))));
    vat::chunk_vat(c, "dummy", &initialize_virt_ctx);
}

criterion_group! {
    name = dummy_read;
    config = Criterion::default()
        .warm_up_time(std::time::Duration::from_millis(300))
        .measurement_time(std::time::Duration::from_millis(2700));
    targets = dummy_read_group
}

criterion_main!(dummy_read);
