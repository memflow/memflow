use criterion::*;

use flow_core::mem::{
    timed_validator::*, CachedMemoryAccess, CachedVirtualTranslate, PageCache, PhysicalMemory,
    TLBCache, VirtualFromPhysical, VirtualMemory, VirtualTranslate,
};

use flow_core::{Address, Length, OsProcessInfo, OsProcessModuleInfo, PageType};

use rand::prelude::*;
use rand::{prng::XorShiftRng as CurRng, Rng, SeedableRng};

fn rwtest<T: VirtualMemory, M: OsProcessModuleInfo>(
    virt_mem: &mut T,
    module: &M,
    chunk_sizes: &[usize],
    chunk_counts: &[usize],
    read_size: usize,
) -> usize {
    let mut rng = CurRng::from_rng(thread_rng()).unwrap();

    let mut total_size = 0;

    for i in chunk_sizes {
        for o in chunk_counts {
            let mut bufs = vec![(Address::null(), vec![0 as u8; *i]); *o];
            let mut done_size = 0;

            while done_size < read_size {
                let base_addr = rng.gen_range(
                    module.base().as_u64(),
                    module.base().as_u64() + module.size().as_u64(),
                );
                for (addr, _) in bufs.iter_mut() {
                    *addr = (base_addr + rng.gen_range(0, 0x2000)).into();
                }

                let _ = virt_mem.virt_read_raw_iter(
                    bufs.iter_mut()
                        .map(|(addr, buf)| (*addr, buf.as_mut_slice())),
                );
                done_size += *i * *o;
            }

            total_size += done_size
        }
    }

    total_size
}

pub fn read_test_with_mem<T: VirtualMemory, M: OsProcessModuleInfo>(
    bench: &mut Bencher,
    virt_mem: &mut T,
    chunk_size: usize,
    chunks: usize,
    tmod: M,
) {
    bench.iter(|| {
        black_box(rwtest(
            virt_mem,
            &tmod,
            &[chunk_size],
            &[chunks],
            chunk_size,
        ));
    });
}

fn read_test_with_ctx<
    T: PhysicalMemory,
    V: VirtualTranslate,
    P: OsProcessInfo,
    M: OsProcessModuleInfo,
>(
    bench: &mut Bencher,
    cache_size: u64,
    chunk_size: usize,
    chunks: usize,
    use_tlb: bool,
    (mut mem, vat, proc, tmod): (T, V, P, M),
) {
    let tlb_cache = TLBCache::new(
        2048.into(),
        TimedCacheValidator::new(Duration::from_millis(1000)),
    );

    if cache_size > 0 {
        let cache = PageCache::new(
            proc.sys_arch(),
            Length::from_mb(cache_size),
            PageType::PAGE_TABLE | PageType::READ_ONLY | PageType::WRITEABLE,
            TimedCacheValidator::new(Duration::from_millis(10000)),
        );

        if use_tlb {
            let mem = CachedMemoryAccess::with(&mut mem, cache);
            let vat = CachedVirtualTranslate::with(vat, tlb_cache, proc.sys_arch());
            let mut virt_mem = VirtualFromPhysical::with_vat(
                mem,
                proc.sys_arch(),
                proc.proc_arch(),
                proc.dtb(),
                vat,
            );
            read_test_with_mem(bench, &mut virt_mem, chunk_size, chunks, tmod);
        } else {
            let mem = CachedMemoryAccess::with(&mut mem, cache);
            let mut virt_mem = VirtualFromPhysical::with_vat(
                mem,
                proc.sys_arch(),
                proc.proc_arch(),
                proc.dtb(),
                vat,
            );
            read_test_with_mem(bench, &mut virt_mem, chunk_size, chunks, tmod);
        }
    } else if use_tlb {
        let vat = CachedVirtualTranslate::with(vat, tlb_cache, proc.sys_arch());
        let mut virt_mem =
            VirtualFromPhysical::with_vat(mem, proc.sys_arch(), proc.proc_arch(), proc.dtb(), vat);
        read_test_with_mem(bench, &mut virt_mem, chunk_size, chunks, tmod);
    } else {
        let mut virt_mem =
            VirtualFromPhysical::with_vat(mem, proc.sys_arch(), proc.proc_arch(), proc.dtb(), vat);
        read_test_with_mem(bench, &mut virt_mem, chunk_size, chunks, tmod);
    }
}

fn seq_read_params<
    T: PhysicalMemory,
    V: VirtualTranslate,
    P: OsProcessInfo,
    M: OsProcessModuleInfo,
>(
    group: &mut BenchmarkGroup<'_, measurement::WallTime>,
    func_name: String,
    cache_size: u64,
    use_tlb: bool,
    initialize_ctx: &dyn Fn() -> flow_core::Result<(T, V, P, M)>,
) {
    for &size in [0x8, 0x10, 0x100, 0x1000, 0x10000].iter() {
        group.throughput(Throughput::Bytes(size));
        group.bench_with_input(
            BenchmarkId::new(func_name.clone(), size),
            &size,
            |b, &size| {
                read_test_with_ctx(
                    b,
                    black_box(cache_size),
                    black_box(size as usize),
                    black_box(1),
                    black_box(use_tlb),
                    initialize_ctx().unwrap(),
                )
            },
        );
    }
}

fn chunk_read_params<
    T: PhysicalMemory,
    V: VirtualTranslate,
    P: OsProcessInfo,
    M: OsProcessModuleInfo,
>(
    group: &mut BenchmarkGroup<'_, measurement::WallTime>,
    func_name: String,
    cache_size: u64,
    use_tlb: bool,
    initialize_ctx: &dyn Fn() -> flow_core::Result<(T, V, P, M)>,
) {
    for &size in [0x8, 0x10, 0x100, 0x1000].iter() {
        for &chunk_size in [1, 4, 16, 64].iter() {
            group.throughput(Throughput::Bytes(size * chunk_size));
            group.bench_with_input(
                BenchmarkId::new(format!("{}_s{:x}", func_name, size), size * chunk_size),
                &size,
                |b, &size| {
                    read_test_with_ctx(
                        b,
                        black_box(cache_size),
                        black_box(size as usize),
                        black_box(chunk_size as usize),
                        black_box(use_tlb),
                        initialize_ctx().unwrap(),
                    )
                },
            );
        }
    }
}

pub fn seq_read<
    T: PhysicalMemory,
    V: VirtualTranslate,
    P: OsProcessInfo,
    M: OsProcessModuleInfo,
>(
    c: &mut Criterion,
    backend_name: &str,
    initialize_ctx: &dyn Fn() -> flow_core::Result<(T, V, P, M)>,
) {
    let plot_config = PlotConfiguration::default().summary_scale(AxisScale::Logarithmic);

    let group_name = format!("{}_virt_seq_read", backend_name);

    let mut group = c.benchmark_group(group_name.clone());
    group.plot_config(plot_config);

    seq_read_params(
        &mut group,
        format!("{}_nocache", group_name),
        0,
        false,
        initialize_ctx,
    );
    seq_read_params(
        &mut group,
        format!("{}_tlb_nocache", group_name),
        0,
        true,
        initialize_ctx,
    );
    seq_read_params(
        &mut group,
        format!("{}_cache", group_name),
        2,
        false,
        initialize_ctx,
    );
    seq_read_params(
        &mut group,
        format!("{}_tlb_cache", group_name),
        2,
        true,
        initialize_ctx,
    );
}

pub fn chunk_read<
    T: PhysicalMemory,
    V: VirtualTranslate,
    P: OsProcessInfo,
    M: OsProcessModuleInfo,
>(
    c: &mut Criterion,
    backend_name: &str,
    initialize_ctx: &dyn Fn() -> flow_core::Result<(T, V, P, M)>,
) {
    let plot_config = PlotConfiguration::default().summary_scale(AxisScale::Logarithmic);

    let group_name = format!("{}_virt_chunk_read", backend_name);

    let mut group = c.benchmark_group(group_name.clone());
    group.plot_config(plot_config);

    chunk_read_params(
        &mut group,
        format!("{}_nocache", group_name),
        0,
        false,
        initialize_ctx,
    );
    chunk_read_params(
        &mut group,
        format!("{}_tlb_nocache", group_name),
        0,
        true,
        initialize_ctx,
    );
    chunk_read_params(
        &mut group,
        format!("{}_cache", group_name),
        2,
        false,
        initialize_ctx,
    );
    chunk_read_params(
        &mut group,
        format!("{}_tlb_cache", group_name),
        2,
        true,
        initialize_ctx,
    );
}
