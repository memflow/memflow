use criterion::*;

use memflow::cglue::as_mut;
use memflow::mem::virt_translate::*;
use memflow::prelude::v1::*;

use rand::prelude::*;
use rand::{Rng, SeedableRng};
use rand_xorshift::XorShiftRng as CurRng;

use std::convert::TryInto;

fn vat_test_with_mem(
    bench: &mut Bencher,
    vat: &mut impl VirtualTranslate,
    chunk_count: usize,
    translations: usize,
    module: ModuleInfo,
) {
    let mut rng = CurRng::from_rng(thread_rng()).unwrap();

    let mut bufs = vec![CTup2(Address::null(), 1); translations];

    let base_addr = rng.gen_range(module.base.to_umem()..(module.base.to_umem() + module.size));

    for CTup2(address, _) in bufs.iter_mut() {
        *address = (base_addr + rng.gen_range(0..0x2000)).into();
    }

    let mut out = vec![];

    bench.iter(|| {
        for chunk in bufs.chunks_mut(chunk_count) {
            out.clear();
            vat.virt_to_phys_list(chunk, (&mut out).into(), (&mut |_| true).into());
            black_box(&out);
        }
    });
}

fn vat_test_with_os(
    bench: &mut Bencher,
    chunks: usize,
    translations: usize,
    os: &mut OsInstanceArcBox<'static>,
) {
    let (mut process, module) = crate::util::find_proc(os).unwrap();

    vat_test_with_mem(
        bench,
        as_mut!(process impl VirtualTranslate).unwrap(),
        chunks,
        translations,
        module,
    );
}

fn chunk_vat_params(
    group: &mut BenchmarkGroup<'_, measurement::WallTime>,
    func_name: String,
    cache_size: usize,
    use_tlb: bool,
    initialize_ctx: &dyn Fn(usize, bool) -> Result<OsInstanceArcBox<'static>>,
) {
    let size = 0x10;

    let mut os = initialize_ctx(cache_size, use_tlb).unwrap();

    for &chunk_size in [1, 4, 16, 64].iter() {
        group.throughput(Throughput::Elements(chunk_size * size));
        group.bench_with_input(
            BenchmarkId::new(func_name.clone(), chunk_size),
            &size,
            |b, &size| {
                vat_test_with_os(
                    b,
                    black_box(chunk_size.try_into().unwrap()),
                    black_box((size * chunk_size).try_into().unwrap()),
                    &mut os,
                )
            },
        );
    }
}

pub fn chunk_vat(
    c: &mut Criterion,
    backend_name: &str,
    initialize_ctx: &dyn Fn(usize, bool) -> Result<OsInstanceArcBox<'static>>,
    use_caches: bool,
) {
    let plot_config = PlotConfiguration::default().summary_scale(AxisScale::Logarithmic);

    let group_name = format!("{backend_name}_chunk_vat");

    let mut group = c.benchmark_group(group_name.clone());
    group.plot_config(plot_config);

    chunk_vat_params(
        &mut group,
        format!("{group_name}_nocache"),
        0,
        false,
        initialize_ctx,
    );
    if use_caches {
        chunk_vat_params(
            &mut group,
            format!("{group_name}_tlb_nocache"),
            0,
            true,
            initialize_ctx,
        );
        chunk_vat_params(
            &mut group,
            format!("{group_name}_cache"),
            2,
            false,
            initialize_ctx,
        );
        chunk_vat_params(
            &mut group,
            format!("{group_name}_tlb_cache"),
            2,
            true,
            initialize_ctx,
        );
    }
}
