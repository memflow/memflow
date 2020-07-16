extern crate memflow_bench;
use memflow_bench::{phys, vat, virt};

use criterion::*;

use memflow_connector::Memory;
use memflow_core::error::{Error, Result};
use memflow_core::mem::TranslateArch;

use memflow_win32::{
    Kernel, KernelInfo, Win32ModuleInfo, Win32Offsets, Win32Process, Win32ProcessInfo,
};

use rand::prelude::*;
use rand::{prng::XorShiftRng as CurRng, Rng, SeedableRng};

fn initialize_virt_ctx() -> Result<(Memory, TranslateArch, Win32ProcessInfo, Win32ModuleInfo)> {
    let mut phys_mem = Memory::new()?;

    let kernel_info = KernelInfo::scanner()
        .mem(&mut phys_mem)
        .scan()
        .map_err(|_| Error::Other("unable to find kernel"))?;
    let vat = TranslateArch::new(kernel_info.start_block.arch);
    let offsets = Win32Offsets::try_with_kernel_info(&kernel_info)
        .map_err(|_| Error::Other("unable to initialize win32 offsets with guid"))?;

    // TODO: remove phys_mem + vat clone
    let mut kernel = Kernel::new(phys_mem.clone(), vat.clone(), offsets, kernel_info);

    let mut rng = CurRng::from_rng(thread_rng()).unwrap();

    let proc_list = kernel
        .process_info_list()
        .map_err(|_| Error::Other("unable to read process list"))?;
    for i in -100..(proc_list.len() as isize) {
        let idx = if i >= 0 {
            i as usize
        } else {
            rng.gen_range(0, proc_list.len())
        };

        let mod_list: Vec<Win32ModuleInfo> = {
            let mut proc = Win32Process::with_kernel(&mut kernel, proc_list[idx].clone());
            proc.module_info_list()
                .unwrap_or_default()
                .into_iter()
                .filter(|module| module.size > 0x1000)
                .collect()
        };

        if !mod_list.is_empty() {
            let tmod = &mod_list[rng.gen_range(0, mod_list.len())];
            return Ok((phys_mem, vat, proc_list[idx].clone(), tmod.clone())); // TODO: remove clone of mem + vat
        }
    }

    Err("No module found!".into())
}

fn win32_read_group(c: &mut Criterion) {
    virt::seq_read(c, "win32", &initialize_virt_ctx);
    virt::chunk_read(c, "win32", &initialize_virt_ctx);
    phys::seq_read(c, "win32", &|| Memory::new());
    phys::chunk_read(c, "win32", &|| Memory::new());
    vat::chunk_vat(c, "win32", &initialize_virt_ctx);
}

criterion_group! {
    name = win32_read;
    config = Criterion::default()
        .warm_up_time(std::time::Duration::from_millis(300))
        .measurement_time(std::time::Duration::from_millis(2700));
    targets = win32_read_group
}

criterion_main!(win32_read);
