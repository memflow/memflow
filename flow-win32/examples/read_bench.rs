extern crate flow_core;
extern crate flow_qemu_procfs;
extern crate flow_win32;
extern crate rand;

use std::io::Write;
use std::time::{Duration, Instant};

use flow_core::{
    timed_validator::*, AccessPhysicalMemory, AccessVirtualMemory, CachedMemoryAccess, PageCache,
};
use flow_core::{Length, OsProcess, OsProcessModule, PageType};
use flow_win32::{Win32, Win32Module, Win32Offsets, Win32Process};

use flow_qemu_procfs::Memory;

use rand::{prng::XorShiftRng as CurRng, Rng, SeedableRng};

fn rwtest<T: AccessVirtualMemory>(
    mem: &mut T,
    proc: &Win32Process,
    module: &dyn OsProcessModule,
    chunk_sizes: &[usize],
    chunk_counts: &[usize],
    read_size: usize,
) {
    let mut rng = CurRng::seed_from_u64(0);

    println!("Performance bench:");
    print!("{:#7}", "SIZE");

    for i in chunk_counts {
        print!(", x{:02x} mb/s, x{:02x} calls/s", *i, *i);
    }

    println!();

    let start = Instant::now();
    let mut ttdur = Duration::new(0, 0);

    for i in chunk_sizes {
        print!("0x{:05x}", *i);
        for o in chunk_counts {
            let mut done_size = 0_usize;
            let mut total_dur = Duration::new(0, 0);
            let mut calls = 0;
            let mut bufs = vec![(vec![0 as u8; *i], 0); *o];

            while done_size < read_size {
                let base_addr = rng.gen_range(
                    module.base().as_u64(),
                    module.base().as_u64() + module.size().as_u64(),
                );
                for (_, addr) in bufs.iter_mut() {
                    *addr = base_addr + rng.gen_range(0, 0x2000);
                }

                let now = Instant::now();
                {
                    let mut vmem = proc.virt_mem(mem);
                    for (buf, addr) in bufs.iter_mut() {
                        let _ = vmem.virt_read_raw_into((*addr).into(), buf.as_mut_slice());
                    }
                }
                total_dur += now.elapsed();
                done_size += *i * *o;
                calls += 1;
            }

            ttdur += total_dur;
            let total_time = total_dur.as_secs_f64();

            print!(
                ", {:8.2}, {:11.2}",
                (done_size / 0x0010_0000) as f64 / total_time,
                calls as f64 / total_time
            );
            std::io::stdout().flush().expect("");
        }
        println!();
    }

    let total_dur = start.elapsed();
    println!(
        "Total bench time: {:.2} {:.2}",
        total_dur.as_secs_f64(),
        ttdur.as_secs_f64()
    );
}

fn read_bench<T: AccessPhysicalMemory + AccessVirtualMemory>(
    mem: &mut T,
    os: Win32,
) -> flow_core::Result<()> {
    let offsets = Win32Offsets::try_with_guid(&os.kernel_guid())?;

    let mut rng = CurRng::seed_from_u64(0);

    let proc_list = os.eprocess_list(mem, &offsets)?;

    loop {
        let proc = Win32Process::try_with_eprocess(
            mem,
            &os,
            &offsets,
            proc_list[rng.gen_range(0, proc_list.len())],
        )?;

        let mod_list: Vec<Win32Module> = proc
            .peb_list(mem)?
            .iter()
            .filter_map(|&x| {
                if let Ok(module) = Win32Module::try_with_peb(mem, &proc, &offsets, x) {
                    if module.size() > 0x1000.into() {
                        Some(module)
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect();

        if !mod_list.is_empty() {
            let tmod = &mod_list[rng.gen_range(0, mod_list.len())];
            println!(
                "Found test module {} ({:x}) in {}",
                tmod.name(),
                tmod.size(),
                proc.name()
            );

            rwtest(
                mem,
                &proc,
                tmod,
                &[0x10000, 0x1000, 0x100, 0x10, 0x8],
                &[32, 8, 1],
                0x0010_0000 * 32,
            );

            break;
        }
    }

    Ok(())
}

fn main() -> flow_core::Result<()> {
    let mut mem_sys = Memory::new()?;
    let os = Win32::try_with(&mut mem_sys)?;

    println!("Benchmarking uncached reads:");
    read_bench(&mut mem_sys, os.clone()).unwrap();

    println!();
    println!("Benchmarking cached reads:");
    let cache = PageCache::new(
        os.start_block.arch,
        Length::from_mb(2),
        PageType::PAGE_TABLE | PageType::READ_ONLY,
        TimedCacheValidator::new(Duration::from_millis(1000).into()),
    );
    let mut mem_cached = CachedMemoryAccess::with(&mut mem_sys, cache);
    read_bench(&mut mem_cached, os).unwrap();

    Ok(())
}
