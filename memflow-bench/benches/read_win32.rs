extern crate memflow_bench;
use memflow_bench::{phys, vat, virt};

use criterion::*;

use memflow::error::{Error, Result};
use memflow::prelude::v1::*;
use memflow_win32::prelude::v1::*;

use rand::prelude::*;
use rand::{Rng, SeedableRng};
use rand_xorshift::XorShiftRng as CurRng;

fn create_connector(args: &ConnectorArgs) -> Result<impl PhysicalMemory> {
    unsafe { memflow::connector::ConnectorInventory::scan().create_connector("qemu_procfs", args) }
}

fn initialize_virt_ctx() -> Result<(
    impl PhysicalMemory,
    DirectTranslate,
    Win32ProcessInfo,
    impl ScopedVirtualTranslate,
    Win32ModuleInfo,
)> {
    let mut phys_mem = create_connector(&ConnectorArgs::new())?;

    let kernel_info = KernelInfo::scanner(&mut phys_mem)
        .scan()
        .map_err(|_| Error::Other("unable to find kernel"))?;
    let mut vat = DirectTranslate::new();
    let offsets = Win32Offsets::builder()
        .kernel_info(&kernel_info)
        .build()
        .map_err(|_| Error::Other("unable to initialize win32 offsets with guid"))?;

    let mut kernel = Kernel::new(&mut phys_mem, &mut vat, offsets, kernel_info);

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
            let mut prc = Win32Process::with_kernel_ref(&mut kernel, proc_list[idx].clone());
            prc.module_list()
                .unwrap_or_default()
                .into_iter()
                .filter(|module| module.size > 0x1000)
                .collect()
        };

        if !mod_list.is_empty() {
            let tmod = &mod_list[rng.gen_range(0, mod_list.len())];
            let proc = proc_list[idx].clone();
            let translator = proc.translator();
            return Ok((phys_mem, vat, proc, translator, tmod.clone())); // TODO: remove clone of mem + vat
        }
    }

    Err("No module found!".into())
}

fn win32_read_group(c: &mut Criterion) {
    virt::seq_read(c, "win32", &initialize_virt_ctx);
    virt::chunk_read(c, "win32", &initialize_virt_ctx);
    phys::seq_read(c, "win32", &|| create_connector(&ConnectorArgs::new()));
    phys::chunk_read(c, "win32", &|| create_connector(&ConnectorArgs::new()));
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
