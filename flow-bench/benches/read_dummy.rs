extern crate flow_bench;
use flow_bench::*;

use criterion::*;

use flow_core::Length;

use flow_core::mem::dummy::{DummyMemory as Memory, DummyModule, DummyProcess};

fn initialize_virt_ctx() -> flow_core::Result<(Memory, DummyProcess, DummyModule)> {
    let mut mem = Memory::new(Length::from_mb(64));

    let proc = mem.alloc_process(Length::from_mb(60), &[]);
    let module = proc.get_module(Length::from_mb(4));

    Ok((mem, proc, module))
}

fn dummy_read_group(c: &mut Criterion) {
    virt::seq_read(c, "dummy", &initialize_virt_ctx);
    virt::chunk_read(c, "dummy", &initialize_virt_ctx);
    phys::seq_read(c, "dummy", &|| Ok(Memory::new(Length::from_mb(64))));
    phys::chunk_read(c, "dummy", &|| Ok(Memory::new(Length::from_mb(64))));
}

criterion_group! {
    name = dummy_read;
    config = Criterion::default()
        .warm_up_time(std::time::Duration::from_millis(300))
        .measurement_time(std::time::Duration::from_millis(2200));
    targets = dummy_read_group
}

criterion_main!(dummy_read);
