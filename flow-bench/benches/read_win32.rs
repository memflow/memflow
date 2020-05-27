extern crate flow_bench;
use flow_bench::{phys, virt};

use criterion::*;

use flow_core::OsProcessModule;

use flow_qemu_procfs::Memory;
use flow_win32::{Win32, Win32Module, Win32Offsets, Win32Process};

use rand::prelude::*;
use rand::{prng::XorShiftRng as CurRng, Rng, SeedableRng};

fn initialize_virt_ctx() -> flow_core::Result<(Memory, Win32Process, Win32Module)> {
    let mut mem = Memory::new().unwrap();

    let os = Win32::try_with(&mut mem).unwrap();
    let offsets = Win32Offsets::try_with_guid(&os.kernel_guid()).unwrap();

    let mut rng = CurRng::from_rng(thread_rng()).unwrap();

    let proc_list = os.eprocess_list(&mut mem, &offsets).unwrap();

    for i in -100..(proc_list.len() as isize) {
        let idx = if i >= 0 {
            i as usize
        } else {
            rng.gen_range(0, proc_list.len())
        };

        if let Ok(proc) = Win32Process::try_with_eprocess(&mut mem, &os, &offsets, proc_list[idx]) {
            let mod_list: Vec<Win32Module> = proc
                .peb_list(&mut mem)
                .unwrap_or_default()
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
                return Ok((mem, proc, tmod.clone()));
            }
        }
    }

    Err("No module found!".into())
}

fn win32_read_group(c: &mut Criterion) {
    virt::seq_read(c, "win32", &initialize_virt_ctx);
    virt::chunk_read(c, "win32", &initialize_virt_ctx);
    phys::seq_read(c, "win32", &|| Memory::new());
    phys::chunk_read(c, "win32", &|| Memory::new());
}

criterion_group! {
    name = win32_read;
    config = Criterion::default()
        .warm_up_time(std::time::Duration::from_millis(300))
        .measurement_time(std::time::Duration::from_millis(2200));
    targets = win32_read_group
}

criterion_main!(win32_read);
