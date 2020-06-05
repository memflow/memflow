extern crate flow_bench;
use flow_bench::*;

use criterion::*;

use flow_core::Length;

use flow_core::architecture::Architecture;
use flow_core::mem::dummy::{DummyMemory as Memory, DummyModule, DummyProcess};
use flow_core::mem::VirtualAdressTranslator;

fn initialize_virt_ctx(
) -> flow_core::Result<(Memory, VirtualAdressTranslator, DummyProcess, DummyModule)> {
    let mut mem = Memory::new(Length::from_mb(64));

    let vat = VirtualAdressTranslator::new(Architecture::X64);

    let proc = mem.alloc_process(Length::from_mb(60), &[]);
    let module = proc.get_module(Length::from_mb(4));
    Ok((mem, vat, proc, module))
}

fn dummy_read_group(c: &mut Criterion) {
    virt::seq_read(c, "dummy", &initialize_virt_ctx);
    virt::chunk_read(c, "dummy", &initialize_virt_ctx);
    phys::seq_read(c, "dummy", &|| Ok(Memory::new(Length::from_mb(64))));
    phys::chunk_read(c, "dummy", &|| Ok(Memory::new(Length::from_mb(64))));
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
