use std::io::Write;
use std::time::{Duration, Instant};

use clap::*;
use log::Level;

use memflow::cglue::*;
use memflow::error::Result;
use memflow::mem::*;
use memflow::os::{ModuleInfo, Os, Process};
use memflow::plugins::*;
use memflow::types::*;

use rand::{Rng, SeedableRng};
use rand_xorshift::XorShiftRng as CurRng;

fn rwtest(
    mut proc: impl Process + MemoryView,
    module: &ModuleInfo,
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
            let mut bufs = vec![(vec![0_u8; *i], 0); *o];

            let base_addr =
                rng.gen_range(module.base.to_umem()..(module.base.to_umem() + module.size));

            // This code will increase the read size for higher number of chunks
            // Since optimized vtop should scale very well with chunk sizes.
            assert!((i.trailing_zeros() as umem) < usize::MAX as umem);
            let chunk_multiplier = *o * (i.trailing_zeros() as usize + 1);

            while done_size < read_size * chunk_multiplier {
                for (_, addr) in bufs.iter_mut() {
                    *addr = base_addr + rng.gen_range(0..0x2000);
                }

                let now = Instant::now();
                {
                    let mut batcher = proc.batcher();

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

fn read_bench(mut kernel: OsInstanceArcBox) -> Result<()> {
    let proc_list = kernel.process_info_list()?;
    let mut rng = CurRng::seed_from_u64(rand::thread_rng().gen_range(0..!0u64));

    let mut cont_cnt = 0usize;

    loop {
        let Ok(mut prc) =
            kernel.process_by_info(proc_list[rng.gen_range(0..proc_list.len())].clone())
        else {
            cont_cnt += 1;
            if cont_cnt.count_ones() == 1 && cont_cnt > 10 {
                println!("Warning: could not get proc {cont_cnt} times in a row");
            }
            continue;
        };

        cont_cnt = 0;

        let mod_list: Vec<ModuleInfo> = prc
            .module_list()?
            .into_iter()
            .filter(|module| module.size > 0x1000)
            .collect();

        if !mod_list.is_empty() {
            let tmod = &mod_list[rng.gen_range(0..mod_list.len())];
            println!(
                "Found test module {} ({:x}) in {}",
                tmod.name,
                tmod.size,
                prc.info().name,
            );

            let mem_map = prc.mapped_mem_vec(smem::gb(1));

            println!("Mapped memory map (with up to 1GB gaps):");

            for CTup3(address, size, pt) in mem_map {
                println!("{:x}-{:x} {:?}", address, address + size, pt);
            }

            rwtest(
                prc,
                tmod,
                &[0x10000, 0x1000, 0x100, 0x10, 0x8],
                &[32, 8, 1],
                0x0010_0000,
            );

            break;
        }
    }

    Ok(())
}

fn main() -> Result<()> {
    let matches = parse_args();
    let (chain, log_level) = extract_args(&matches)?;

    simplelog::TermLogger::init(
        log_level.to_level_filter(),
        simplelog::Config::default(),
        simplelog::TerminalMode::Stdout,
        simplelog::ColorChoice::Auto,
    )
    .unwrap();

    // create connector + os
    let inventory = Inventory::scan();

    let os = inventory.builder().os_chain(chain).build()?;

    read_bench(os)
}

fn parse_args() -> ArgMatches {
    Command::new("read_bench example")
        .version(crate_version!())
        .author(crate_authors!())
        .arg(Arg::new("verbose").short('v').action(ArgAction::Count))
        .arg(
            Arg::new("connector")
                .long("connector")
                .short('c')
                .action(ArgAction::Append)
                .required(false),
        )
        .arg(
            Arg::new("os")
                .long("os")
                .short('o')
                .action(ArgAction::Append)
                .required(true),
        )
        .get_matches()
}

fn extract_args(matches: &ArgMatches) -> Result<(OsChain<'_>, log::Level)> {
    // set log level
    let level = match matches.get_count("verbose") {
        0 => Level::Error,
        1 => Level::Warn,
        2 => Level::Info,
        3 => Level::Debug,
        4 => Level::Trace,
        _ => Level::Trace,
    };

    let conn_iter = matches
        .indices_of("connector")
        .zip(matches.get_many::<String>("connector"))
        .map(|(a, b)| a.zip(b.map(String::as_str)))
        .into_iter()
        .flatten();

    let os_iter = matches
        .indices_of("os")
        .zip(matches.get_many::<String>("os"))
        .map(|(a, b)| a.zip(b.map(String::as_str)))
        .into_iter()
        .flatten();

    Ok((OsChain::new(conn_iter, os_iter)?, level))
}
