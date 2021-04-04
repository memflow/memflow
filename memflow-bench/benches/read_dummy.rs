extern crate memflow_bench;
use memflow_bench::*;

use criterion::*;

use memflow::dummy::DummyMemory as Memory;
use memflow::dummy::DummyOS;
use memflow::os::Process;
use memflow::prelude::v1::*;

#[allow(clippy::unnecessary_wraps)]
fn initialize_virt_ctx() -> Result<(
    Memory,
    DirectTranslate,
    ProcessInfo,
    impl ScopedVirtualTranslate,
    ModuleInfo,
)> {
    let mem = Memory::new(size::mb(64));
    let mut os = DummyOS::new(mem);

    let vat = DirectTranslate::new();

    let pid = os.alloc_process(size::mb(60), &[]);
    let mut prc = os.process_by_pid(pid).unwrap();
    let module = prc.primary_module().unwrap();
    let translator = prc.proc.translator();
    let info = prc.proc.info;
    Ok((os.into_inner(), vat, info, translator, module))
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
