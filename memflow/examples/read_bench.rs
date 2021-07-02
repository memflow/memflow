use std::io::Write;
use std::time::{Duration, Instant};

use clap::*;
use log::Level;

use memflow::cglue::*;
use memflow::error::{Error, ErrorKind, ErrorOrigin, Result};
use memflow::mem::*;
use memflow::os::{ModuleInfo, OsInner, OsInstanceArcBox, Process};
use memflow::plugins::*;
use memflow::types::*;

use rand::{Rng, SeedableRng};
use rand_xorshift::XorShiftRng as CurRng;

fn rwtest(
    mut proc: impl Process + VirtualMemory,
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
                rng.gen_range(module.base.as_u64()..(module.base.as_u64() + module.size as u64));

            // This code will increase the read size for higher number of chunks
            // Since optimized vtop should scale very well with chunk sizes.
            let chunk_multiplier = *o * (i.trailing_zeros() as usize + 1);

            while done_size < read_size * chunk_multiplier {
                for (_, addr) in bufs.iter_mut() {
                    *addr = base_addr + rng.gen_range(0..0x2000);
                }

                let now = Instant::now();
                {
                    let mut batcher = proc.virt_batcher();

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
    loop {
        let mut prc =
            kernel.process_by_info(proc_list[rng.gen_range(0..proc_list.len())].clone())?;

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

            let mem_map = {
                let prc = as_mut!(prc impl VirtualTranslate)
                    .ok_or(ErrorKind::UnsupportedOptionalFeature)?;
                prc.virt_page_map_vec(size::gb(1))
            };

            println!("Memory map (with up to 1GB gaps):");

            for map in mem_map {
                println!("{:x}-{:x}", map.address, map.address + map.size);
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
    let (conn_name, conn_args, os_name, os_args, log_level) = parse_args()?;

    simple_logger::SimpleLogger::new()
        .with_level(log_level.to_level_filter())
        .init()
        .unwrap();

    // create connector + os
    let inventory = Inventory::scan();
    let os = inventory
        .builder()
        .connector(&conn_name)
        .args(conn_args)
        .os(&os_name)
        .args(os_args)
        .build()?;

    read_bench(os)
}

fn parse_args() -> Result<(String, Args, String, Args, log::Level)> {
    let matches = App::new("read_bench example")
        .version(crate_version!())
        .author(crate_authors!())
        .arg(Arg::with_name("verbose").short("v").multiple(true))
        .arg(
            Arg::with_name("connector")
                .long("connector")
                .short("c")
                .takes_value(true)
                .required(false),
        )
        .arg(
            Arg::with_name("conn-args")
                .long("conn-args")
                .short("x")
                .takes_value(true)
                .default_value(""),
        )
        .arg(
            Arg::with_name("os")
                .long("os")
                .short("o")
                .takes_value(true)
                .required(true),
        )
        .arg(
            Arg::with_name("os-args")
                .long("os-args")
                .short("y")
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

    Ok((
        matches.value_of("connector").unwrap_or("").into(),
        Args::parse(matches.value_of("conn-args").ok_or_else(|| {
            Error(ErrorOrigin::Other, ErrorKind::Configuration)
                .log_error("failed to parse connector args")
        })?)?,
        matches
            .value_of("os")
            .ok_or_else(|| {
                Error(ErrorOrigin::Other, ErrorKind::Configuration).log_error("failed to parse os")
            })?
            .into(),
        Args::parse(matches.value_of("os-args").ok_or_else(|| {
            Error(ErrorOrigin::Other, ErrorKind::Configuration).log_error("failed to parse os args")
        })?)?,
        level,
    ))
}
