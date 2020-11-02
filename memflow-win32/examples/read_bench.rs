use std::io::Write;
use std::time::{Duration, Instant};

use clap::*;
use log::Level;

use memflow::connector::*;
use memflow::mem::*;
use memflow::process::*;
use memflow::types::*;

use memflow_win32::error::Result;
use memflow_win32::offsets::Win32Offsets;
use memflow_win32::win32::{Kernel, KernelInfo, Win32ModuleInfo, Win32Process};

use rand::{Rng, SeedableRng};
use rand_xorshift::XorShiftRng as CurRng;

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

fn read_bench<T: PhysicalMemory + ?Sized, V: VirtualTranslate>(
    phys_mem: &mut T,
    vat: &mut V,
    kernel_info: KernelInfo,
) -> Result<()> {
    let offsets = Win32Offsets::builder().kernel_info(&kernel_info).build()?;
    let mut kernel = Kernel::new(phys_mem, vat, offsets, kernel_info);

    let proc_list = kernel.process_info_list()?;
    let mut rng = CurRng::seed_from_u64(rand::thread_rng().gen_range(0, !0u64));
    loop {
        let mut prc = Win32Process::with_kernel_ref(
            &mut kernel,
            proc_list[rng.gen_range(0, proc_list.len())].clone(),
        );

        let mod_list: Vec<Win32ModuleInfo> = prc
            .module_list()?
            .into_iter()
            .filter(|module| module.size() > 0x1000)
            .collect();

        if !mod_list.is_empty() {
            let tmod = &mod_list[rng.gen_range(0, mod_list.len())];
            println!(
                "Found test module {} ({:x}) in {}",
                tmod.name(),
                tmod.size(),
                prc.proc_info.name(),
            );

            let mem_map = prc.virt_mem.virt_page_map(size::gb(1));

            println!("Memory map (with up to 1GB gaps):");

            for (addr, len) in mem_map {
                println!("{:x}-{:x}", addr, addr + len);
            }

            rwtest(
                &mut prc,
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
    let matches = App::new("read_keys example")
        .version(crate_version!())
        .author(crate_authors!())
        .arg(Arg::with_name("verbose").short("v").multiple(true))
        .arg(
            Arg::with_name("connector")
                .long("connector")
                .short("c")
                .takes_value(true)
                .required(true),
        )
        .arg(
            Arg::with_name("args")
                .long("args")
                .short("a")
                .takes_value(true)
                .default_value(""),
        )
        .get_matches();

    // set log level
    let level = match matches.occurrences_of("verbose") {
        0 => Level::Error,
        1 => Level::Warn,
        2 => Level::Info,
        3 => Level::Debug,
        4 => Level::Trace,
        _ => Level::Trace,
    };
    simple_logger::SimpleLogger::new()
        .with_level(level.to_level_filter())
        .init()
        .unwrap();

    // create inventory + connector
    let inventory = unsafe { ConnectorInventory::scan() };
    let mut connector = unsafe {
        inventory.create_connector(
            matches.value_of("connector").unwrap(),
            &ConnectorArgs::parse(matches.value_of("args").unwrap()).unwrap(),
        )
    }
    .unwrap();

    // scan for win32 kernel
    let kernel_info = KernelInfo::scanner(&mut connector).scan()?;

    let mut vat = DirectTranslate::new();

    println!("Benchmarking uncached reads:");
    read_bench(&mut connector, &mut vat, kernel_info.clone()).unwrap();

    println!();
    println!("Benchmarking cached reads:");
    let mut mem_cached = CachedMemoryAccess::builder(&mut connector)
        .arch(kernel_info.start_block.arch)
        .build()
        .unwrap();

    let mut vat_cached = CachedVirtualTranslate::builder(vat)
        .arch(kernel_info.start_block.arch)
        .build()
        .unwrap();

    read_bench(&mut mem_cached, &mut vat_cached, kernel_info).unwrap();

    println!("TLB Hits {}\nTLB Miss {}", vat_cached.hitc, vat_cached.misc);

    Ok(())
}
