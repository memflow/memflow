use criterion::*;

use memflow::mem::{CachedMemoryAccess, CachedVirtualTranslate, PhysicalMemory, VirtualTranslate};

use memflow::architecture::ScopedVirtualTranslate;

use memflow::error::Result;
use memflow::iter::FnExtend;
use memflow::os::*;
use memflow::types::*;

use rand::prelude::*;
use rand::{Rng, SeedableRng};
use rand_xorshift::XorShiftRng as CurRng;

fn vat_test_with_mem<T: PhysicalMemory, V: VirtualTranslate, S: ScopedVirtualTranslate>(
    bench: &mut Bencher,
    phys_mem: &mut T,
    vat: &mut V,
    chunk_count: usize,
    translations: usize,
    translator: S,
    module: ModuleInfo,
) {
    let mut rng = CurRng::from_rng(thread_rng()).unwrap();

    let mut bufs = vec![Address::null(); translations];

    let base_addr = rng.gen_range(
        module.base.as_u64(),
        module.base.as_u64() + module.size as u64,
    );

    for addr in bufs.iter_mut() {
        *addr = (base_addr + rng.gen_range(0, 0x2000)).into();
    }

    let mut out = Vec::new();

    bench.iter(|| {
        for chunk in bufs.chunks_mut(chunk_count) {
            out.clear();
            vat.virt_to_phys_iter(
                phys_mem,
                &translator,
                chunk.iter_mut().map(|x| (*x, 1)),
                &mut out,
                &mut FnExtend::new(|_| {}),
            );
            black_box(&out);
        }
    });
}

fn vat_test_with_ctx<T: PhysicalMemory, V: VirtualTranslate, S: ScopedVirtualTranslate>(
    bench: &mut Bencher,
    cache_size: u64,
    chunks: usize,
    translations: usize,
    use_tlb: bool,
    (mut mem, mut vat, prc, translator, tmod): (T, V, ProcessInfo, S, ModuleInfo),
) {
    if cache_size > 0 {
        let cache = CachedMemoryAccess::builder(&mut mem)
            .arch(prc.sys_arch)
            .cache_size(size::mb(cache_size as usize))
            .page_type_mask(PageType::PAGE_TABLE | PageType::READ_ONLY | PageType::WRITEABLE);

        if use_tlb {
            let mut mem = cache.build().unwrap();
            let mut vat = CachedVirtualTranslate::builder(vat)
                .arch(prc.sys_arch)
                .build()
                .unwrap();
            vat_test_with_mem(
                bench,
                &mut mem,
                &mut vat,
                chunks,
                translations,
                translator,
                tmod,
            );
        } else {
            let mut mem = cache.build().unwrap();
            vat_test_with_mem(
                bench,
                &mut mem,
                &mut vat,
                chunks,
                translations,
                translator,
                tmod,
            );
        }
    } else if use_tlb {
        let mut vat = CachedVirtualTranslate::builder(vat)
            .arch(prc.sys_arch)
            .build()
            .unwrap();
        vat_test_with_mem(
            bench,
            &mut mem,
            &mut vat,
            chunks,
            translations,
            translator,
            tmod,
        );
    } else {
        vat_test_with_mem(
            bench,
            &mut mem,
            &mut vat,
            chunks,
            translations,
            translator,
            tmod,
        );
    }
}

fn chunk_vat_params<T: PhysicalMemory, V: VirtualTranslate, S: ScopedVirtualTranslate>(
    group: &mut BenchmarkGroup<'_, measurement::WallTime>,
    func_name: String,
    cache_size: u64,
    use_tlb: bool,
    initialize_ctx: &dyn Fn() -> Result<(T, V, ProcessInfo, S, ModuleInfo)>,
) {
    let size = 0x10;
    for &chunk_size in [1, 4, 16, 64].iter() {
        group.throughput(Throughput::Elements(chunk_size * size));
        group.bench_with_input(
            BenchmarkId::new(func_name.clone(), chunk_size),
            &size,
            |b, &size| {
                vat_test_with_ctx(
                    b,
                    black_box(cache_size),
                    black_box(chunk_size as usize),
                    black_box((size * chunk_size) as usize),
                    black_box(use_tlb),
                    initialize_ctx().unwrap(),
                )
            },
        );
    }
}

pub fn chunk_vat<T: PhysicalMemory, V: VirtualTranslate, S: ScopedVirtualTranslate>(
    c: &mut Criterion,
    backend_name: &str,
    initialize_ctx: &dyn Fn() -> Result<(T, V, ProcessInfo, S, ModuleInfo)>,
) {
    let plot_config = PlotConfiguration::default().summary_scale(AxisScale::Logarithmic);

    let group_name = format!("{}_chunk_vat", backend_name);

    let mut group = c.benchmark_group(group_name.clone());
    group.plot_config(plot_config);

    chunk_vat_params(
        &mut group,
        format!("{}_nocache", group_name),
        0,
        false,
        initialize_ctx,
    );
    chunk_vat_params(
        &mut group,
        format!("{}_tlb_nocache", group_name),
        0,
        true,
        initialize_ctx,
    );
    chunk_vat_params(
        &mut group,
        format!("{}_cache", group_name),
        2,
        false,
        initialize_ctx,
    );
    chunk_vat_params(
        &mut group,
        format!("{}_tlb_cache", group_name),
        2,
        true,
        initialize_ctx,
    );
}
