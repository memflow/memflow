extern crate memflow_connector;
extern crate memflow_core;
extern crate memflow_win32;
extern crate rand;

use std::io::Write;
use std::time::{Duration, Instant};

use memflow_core::mem::cache::{CachedMemoryAccess, CachedVirtualTranslate, TimedCacheValidator};
use memflow_core::mem::{PhysicalMemory, TranslateArch, VirtualMemory, VirtualTranslate};
use memflow_core::process::{OsProcessInfo, OsProcessModuleInfo};
use memflow_core::types::{size, Address};

use memflow_connector::create_connector;

use memflow_win32::error::Result;
use memflow_win32::offsets::Win32Offsets;
use memflow_win32::win32::{Kernel, KernelInfo, Win32ModuleInfo, Win32Process};

use rand::{prng::XorShiftRng as CurRng, Rng, SeedableRng};

fn rwtest<T: VirtualMemory>(
    proc: &mut Win32Process<T>,
    module: &dyn OsProcessModuleInfo,
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

            let base_addr = rng.gen_range(
                module.base().as_u64(),
                module.base().as_u64() + module.size() as u64,
            );

            while done_size < read_size {
                for (_, addr) in bufs.iter_mut() {
                    *addr = base_addr + rng.gen_range(0, 0x2000);
                }

                let now = Instant::now();
                {
                    let mut batcher = proc.virt_mem.virt_batcher();

                    for (buf, addr) in bufs.iter_mut() {
                        batcher.read_raw_into(Address::from(*addr), buf);
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

fn read_bench<T: PhysicalMemory, V: VirtualTranslate>(
    phys_mem: &mut T,
    vat: &mut V,
    kernel_info: KernelInfo,
) -> Result<()> {
    let offsets = Win32Offsets::try_with_kernel_info(&kernel_info)?;
    let mut kernel = Kernel::new(phys_mem, vat, offsets, kernel_info);

    let proc_list = kernel.process_info_list()?;
    let mut rng = CurRng::seed_from_u64(rand::thread_rng().gen_range(0, !0u64));
    loop {
        let mut proc = Win32Process::with_kernel(
            &mut kernel,
            proc_list[rng.gen_range(0, proc_list.len())].clone(),
        );

        let mod_list: Vec<Win32ModuleInfo> = proc
            .module_info_list()?
            .into_iter()
            .filter(|module| module.size() > 0x1000)
            .collect();

        if !mod_list.is_empty() {
            let tmod = &mod_list[rng.gen_range(0, mod_list.len())];
            println!(
                "Found test module {} ({:x}) in {}",
                tmod.name(),
                tmod.size(),
                proc.proc_info.name(),
            );

            let mem_map = proc.virt_mem.virt_page_map(size::gb(1));

            println!("Memory map (with up to 1GB gaps):");

            for (addr, len) in mem_map {
                println!("{:x}-{:x}", addr, addr + len);
            }

            rwtest(
                &mut proc,
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

fn main() -> Result<()> {
    let mut mem_sys = create_connector("")?;
    let kernel_info = KernelInfo::scanner().mem(&mut mem_sys).scan()?;

    let mut vat = TranslateArch::new(kernel_info.start_block.arch);

    println!("Benchmarking uncached reads:");
    read_bench(&mut mem_sys, &mut vat, kernel_info.clone()).unwrap();

    println!();
    println!("Benchmarking cached reads:");
    let mut mem_cached = CachedMemoryAccess::builder()
        .mem(mem_sys)
        .arch(kernel_info.start_block.arch)
        .validator(TimedCacheValidator::new(Duration::from_millis(1000).into()))
        .build()
        .unwrap();

    let mut vat_cached = CachedVirtualTranslate::builder()
        .vat(vat)
        .arch(kernel_info.start_block.arch)
        .validator(TimedCacheValidator::new(Duration::from_millis(1000).into()))
        .build()
        .unwrap();

    read_bench(&mut mem_cached, &mut vat_cached, kernel_info).unwrap();

    println!("TLB Hits {}\nTLB Miss {}", vat_cached.hitc, vat_cached.misc);

    Ok(())
}
