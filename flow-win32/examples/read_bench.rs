extern crate flow_core;
extern crate flow_qemu_procfs;
extern crate flow_win32;
extern crate rand;

use flow_core::mem::{cache::TimedCache, PageType};
use flow_core::{Length, OsProcess, OsProcessModule};
use flow_qemu_procfs::Memory;
use flow_win32::{Win32, Win32Module, Win32Offsets, Win32Process};
use rand::{prng::XorShiftRng as CurRng, Rng, SeedableRng};
use std::io::Write;
use std::time::{Duration, Instant};

fn rwtest(
    mem: &mut Memory<TimedCache>,
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

fn main() -> flow_core::Result<()> {
    let mut mem = Memory::new(TimedCache::new(
        100,
        0x200,
        Length::from_kb(4),
        PageType::READ_ONLY | PageType::PAGE_TABLE,
    ))?;
    let os = Win32::try_with(&mut mem)?;
    let offsets = Win32Offsets::try_with_guid(&os.kernel_guid())?;

    let mut rng = CurRng::seed_from_u64(0);

    let proc_list = os.eprocess_list(&mut mem, &offsets)?;

    loop {
        let proc = Win32Process::try_with_eprocess(
            &mut mem,
            &os,
            &offsets,
            proc_list[rng.gen_range(0, proc_list.len())],
        )?;

        let mod_list: Vec<Win32Module> = proc
            .peb_list(&mut mem)?
            .iter()
            .filter_map(|&x| {
                if let Ok(module) = Win32Module::try_with_peb(&mut mem, &proc, &offsets, x) {
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
                &mut mem,
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
