
use criterion::*;

use flow_core::mem::{
    timed_validator::*, vat::VirtualAddressTranslator, AccessVirtualMemory, AccessPhysicalMemory, CachedMemoryAccess, CachedVAT, PageCache, TLBCache,
};

use flow_core::{Address, Length, OsProcess, OsProcessModule, PageType};

use rand::prelude::*;
use rand::{prng::XorShiftRng as CurRng, Rng, SeedableRng};

pub fn rwtest<T: AccessVirtualMemory, P: OsProcess, M: OsProcessModule>(
    mem: &mut T,
    proc: &P,
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

                {
                    let mut vmem = proc.virt_mem(mem);
                    for (addr, buf) in bufs.iter_mut() {
                        let _ = vmem.virt_read_raw_into(*addr, buf.as_mut_slice());
                    }
                }
                done_size += *i * *o;
            }

            total_size += done_size
        }
    }

    total_size
}

pub fn read_test_with_mem<T: AccessVirtualMemory, P: OsProcess, M: OsProcessModule>(
    bench: &mut Bencher,
    mem: &mut T,
    chunk_size: usize,
    chunks: usize,
    proc: P,
    tmod: M,
) {
    bench.iter(|| {
        black_box(rwtest(
            mem,
            &proc,
            &tmod,
            &[chunk_size],
            &[chunks],
            chunk_size,
        ));
    });
}

pub fn read_test_with_ctx<T: VirtualAddressTranslator + AccessVirtualMemory + AccessPhysicalMemory, P: OsProcess, M: OsProcessModule>(
    bench: &mut Bencher,
    cache_size: u64,
    chunk_size: usize,
    chunks: usize,
    use_tlb: bool,
    mut mem: T,
    proc: P,
    tmod: M
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
            read_test_with_mem(bench, &mut mem, chunk_size, chunks, proc, tmod);
        } else {
            let mut mem = CachedMemoryAccess::with(&mut mem, cache);
            read_test_with_mem(bench, &mut mem, chunk_size, chunks, proc, tmod);
        }
    } else if use_tlb {
        let mut mem = CachedVAT::with(mem, tlb_cache);
        read_test_with_mem(bench, &mut mem, chunk_size, chunks, proc, tmod);
    } else {
        read_test_with_mem(bench, &mut mem, chunk_size, chunks, proc, tmod);
    }
}

