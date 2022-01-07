extern crate memflow_bench;
use memflow_bench::{phys, util, vat, virt};

use criterion::*;

use memflow::prelude::v1::*;

fn create_connector(args: Option<&ConnectorArgs>) -> Result<impl PhysicalMemory + Clone> {
    // this workaround is to prevent loaded libraries
    // from spitting out to much log information and skewing benchmarks
    let filter = log::max_level();
    log::set_max_level(log::Level::Error.to_level_filter());

    let result = Inventory::scan().create_connector("qemu", None, args)?;

    log::set_max_level(filter);
    Ok(result)
}

fn initialize_virt_ctx(cache_size: usize, use_tlb: bool) -> Result<OsInstanceArcBox<'static>> {
    util::build_os("qemu", cache_size, "win32", use_tlb)
}

fn win32_read_group(c: &mut Criterion) {
    virt::seq_read(c, "win32", &initialize_virt_ctx, true);
    virt::chunk_read(c, "win32", &initialize_virt_ctx, true);
    phys::seq_read(c, "win32", &|| create_connector(None));
    phys::chunk_read(c, "win32", &|| create_connector(None));
    vat::chunk_vat(c, "win32", &initialize_virt_ctx, true);
}

criterion_group! {
    name = win32_read;
    config = Criterion::default()
        .warm_up_time(std::time::Duration::from_millis(1000))
        .measurement_time(std::time::Duration::from_millis(10000));
    targets = win32_read_group
}

criterion_main!(win32_read);
