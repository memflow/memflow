use criterion::*;

use flow_core::mem::{
    timed_validator::*, vat::VirtualAddressTranslatorRaw, AccessPhysicalMemoryRaw,
    CachedMemoryAccess, CachedVAT, PageCache, TLBCache,
};

use flow_core::{Address, Length, OsProcess, OsProcessModule, PageType};

use rand::prelude::*;
use rand::{prng::XorShiftRng as CurRng, Rng, SeedableRng};

fn vattest<T: VirtualAddressTranslatorRaw, P: OsProcess, M: OsProcessModule>(
    mem: &mut T,
    proc: &P,
    module: &M,
    chunk_count: usize,
    translations: usize,
) -> usize {
    let mut rng = CurRng::from_rng(thread_rng()).unwrap();

    let mut bufs = vec![Address::null(); chunk_count];
    let mut done_size = 0;

    let mut out = Vec::new();

    while done_size < translations {
        let base_addr = rng.gen_range(
            module.base().as_u64(),
            module.base().as_u64() + module.size().as_u64(),
        );

        for addr in bufs.iter_mut() {
            *addr = (base_addr + rng.gen_range(0, 0x2000)).into();
        }

        out.clear();
        black_box(mem.virt_to_phys_iter(
            proc.sys_arch(),
            proc.dtb(),
            bufs.iter_mut().map(|x| (*x, Option::<&[u8]>::None)),
            &mut out,
        ));

        done_size += chunk_count;
    }

    done_size
}

pub fn vat_test_with_mem<T: VirtualAddressTranslatorRaw, P: OsProcess, M: OsProcessModule>(
    bench: &mut Bencher,
    mem: &mut T,
    chunks: usize,
    translations: usize,
    proc: P,
    tmod: M,
) {
    bench.iter(|| {
        black_box(vattest(mem, &proc, &tmod, chunks, translations));
    });
}

fn vat_test_with_ctx<
    T: VirtualAddressTranslatorRaw + AccessPhysicalMemoryRaw,
    P: OsProcess,
    M: OsProcessModule,
>(
    bench: &mut Bencher,
    cache_size: u64,
    chunks: usize,
    translations: usize,
    use_tlb: bool,
    (mut mem, proc, tmod): (T, P, M),
) {
    let tlb_cache = TLBCache::new(
        2048.into(),
        TimedCacheValidator::new(Duration::from_millis(1000).into()),
    );

    if cache_size > 0 {
        let cache = PageCache::new(
            proc.sys_arch(),
            Length::from_mb(cache_size),
            PageType::PAGE_TABLE | PageType::READ_ONLY | PageType::WRITEABLE,
            TimedCacheValidator::new(Duration::from_millis(10000).into()),
        );

        if use_tlb {
            let mem = CachedMemoryAccess::with(&mut mem, cache);
            let mut mem = CachedVAT::with(mem, tlb_cache);
            vat_test_with_mem(bench, &mut mem, chunks, translations, proc, tmod);
        } else {
            let mut mem = CachedMemoryAccess::with(&mut mem, cache);
            vat_test_with_mem(bench, &mut mem, chunks, translations, proc, tmod);
        }
    } else if use_tlb {
        let mut mem = CachedVAT::with(mem, tlb_cache);
        vat_test_with_mem(bench, &mut mem, chunks, translations, proc, tmod);
    } else {
        vat_test_with_mem(bench, &mut mem, chunks, translations, proc, tmod);
    }
}

fn chunk_vat_params<
    T: VirtualAddressTranslatorRaw + AccessPhysicalMemoryRaw,
    P: OsProcess,
    M: OsProcessModule,
>(
    group: &mut BenchmarkGroup<'_, measurement::WallTime>,
    func_name: String,
    cache_size: u64,
    use_tlb: bool,
    initialize_ctx: &dyn Fn() -> flow_core::Result<(T, P, M)>,
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

pub fn chunk_vat<
    T: VirtualAddressTranslatorRaw + AccessPhysicalMemoryRaw,
    P: OsProcess,
    M: OsProcessModule,
>(
    c: &mut Criterion,
    backend_name: &str,
    initialize_ctx: &dyn Fn() -> flow_core::Result<(T, P, M)>,
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
