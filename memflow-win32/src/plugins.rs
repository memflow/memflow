use crate::offsets::SymbolStore;
use crate::win32::{Win32Kernel, Win32KernelBuilder};
use memflow::architecture::ArchitectureIdent;
use memflow::derive::*;
use memflow::error::*;
use memflow::mem::cache::TimedCacheValidator;
use memflow::mem::cache::{CachedMemoryAccess, CachedVirtualTranslate};
use memflow::mem::{PhysicalMemory, VirtualTranslate};
use memflow::plugins::{Args, ConnectorInstance, OSInstance};
use memflow::types::{size, Address};
use std::time::Duration;

#[os_layer_bare(name = "win32")]
pub fn build_kernel(
    args: &Args,
    mem: ConnectorInstance,
    log_level: log::Level,
) -> Result<OSInstance> {
    simple_logger::SimpleLogger::new()
        .with_level(log_level.to_level_filter())
        .init()
        .ok();

    let builder = Win32Kernel::builder(mem);

    build_dtb(builder, args)
}

fn build_final<
    A: 'static + PhysicalMemory + Clone,
    B: 'static + PhysicalMemory + Clone,
    C: 'static + VirtualTranslate + Clone,
>(
    builder: Win32KernelBuilder<A, B, C>,
    _: &Args,
) -> Result<OSInstance> {
    log::info!(
        "Building kernel of type {}",
        std::any::type_name::<Win32KernelBuilder<A, B, C>>()
    );
    builder.build().map_err(From::from).map(OSInstance::new)
}

fn build_arch<
    A: 'static + PhysicalMemory + Clone,
    B: 'static + PhysicalMemory + Clone,
    C: 'static + VirtualTranslate + Clone,
>(
    builder: Win32KernelBuilder<A, B, C>,
    args: &Args,
) -> Result<OSInstance> {
    match args.get("arch").map(|a| a.to_lowercase()).as_deref() {
        Some("x64") => build_final(builder.arch(ArchitectureIdent::X86(64, false)), args),
        Some("x32") => build_final(builder.arch(ArchitectureIdent::X86(32, false)), args),
        Some("x32_pae") => build_final(builder.arch(ArchitectureIdent::X86(32, true)), args),
        Some("aarch64") => build_final(builder.arch(ArchitectureIdent::AArch64(size::kb(4))), args),
        _ => build_final(builder, args),
    }
}

fn build_symstore<
    A: 'static + PhysicalMemory + Clone,
    B: 'static + PhysicalMemory + Clone,
    C: 'static + VirtualTranslate + Clone,
>(
    builder: Win32KernelBuilder<A, B, C>,
    args: &Args,
) -> Result<OSInstance> {
    match args.get("symstore") {
        Some("uncached") => build_arch(builder.symbol_store(SymbolStore::new().no_cache()), args),
        Some("none") => build_arch(builder.no_symbol_store(), args),
        _ => build_arch(builder, args),
    }
}

fn build_kernel_hint<
    A: 'static + PhysicalMemory + Clone,
    B: 'static + PhysicalMemory + Clone,
    C: 'static + VirtualTranslate + Clone,
>(
    builder: Win32KernelBuilder<A, B, C>,
    args: &Args,
) -> Result<OSInstance> {
    match args
        .get("kernel_hint")
        .and_then(|d| u64::from_str_radix(d, 16).ok())
    {
        Some(dtb) => build_symstore(builder.kernel_hint(Address::from(dtb)), args),
        _ => build_symstore(builder, args),
    }
}

fn build_page_cache<
    A: 'static + PhysicalMemory + Clone,
    B: 'static + PhysicalMemory + Clone,
    C: 'static + VirtualTranslate + Clone,
>(
    builder: Win32KernelBuilder<A, B, C>,
    mode: &str,
    args: &Args,
) -> Result<OSInstance> {
    match mode.split('&').find(|s| s.contains("page")) {
        Some(page) => match page.split(':').nth(1) {
            Some(vargs) => {
                let mut sp = vargs.splitn(2, ';');
                let (size, time) = (
                    sp.next()
                        .ok_or(Error::Other("Failed to parse Page Cache size"))?,
                    sp.next()
                        .ok_or(Error::Other("Failed to parse Page Cache validator time"))?,
                );

                let (size, size_mul) = {
                    let mul_arr = &[
                        (size::kb(1), "kb"),
                        (size::kb(1), "KB"),
                        (size::kb(1), "k"),
                        (size::kb(1), "K"),
                        (size::mb(1), "mb"),
                        (size::mb(1), "MB"),
                        (size::mb(1), "m"),
                        (size::mb(1), "M"),
                        (size::gb(1), "gb"),
                        (size::gb(1), "GB"),
                        (size::gb(1), "m"),
                        (size::gb(1), "G"),
                    ];

                    mul_arr
                        .iter()
                        .filter_map(|(m, e)| {
                            if size.ends_with(e) {
                                Some((size.trim_end_matches(e), *m))
                            } else {
                                None
                            }
                        })
                        .next()
                        .ok_or(Error::Other("Invalid Page Cache size unit (or none)!"))?
                };

                let size = usize::from_str_radix(size, 16)
                    .map_err(|_| Error::Other("Failed to parse Page Cache size"))?;

                let size = size * size_mul;

                let time = u64::from_str_radix(time, 10)
                    .map_err(|_| Error::Other("Failed to parse Page Cache validity time"))?;
                build_kernel_hint(
                    builder.build_page_cache(move |v, a| {
                        CachedMemoryAccess::builder(v)
                            .arch(a)
                            .cache_size(size)
                            .validator(TimedCacheValidator::new(Duration::from_millis(time).into()))
                            .build()
                            .unwrap()
                    }),
                    args,
                )
            }
            None => build_kernel_hint(
                builder.build_page_cache(|v, a| {
                    CachedMemoryAccess::builder(v).arch(a).build().unwrap()
                }),
                args,
            ),
        },
        None => build_kernel_hint(builder, args),
    }
}

fn build_vat<
    A: 'static + PhysicalMemory + Clone,
    B: 'static + PhysicalMemory + Clone,
    C: 'static + VirtualTranslate + Clone,
>(
    builder: Win32KernelBuilder<A, B, C>,
    mode: &str,
    args: &Args,
) -> Result<OSInstance> {
    match mode.split('&').find(|s| s.contains("vat")) {
        Some(vat) => match vat.split(':').nth(1) {
            Some(vargs) => {
                let mut sp = vargs.splitn(2, ';');
                let (size, time) = (
                    sp.next().ok_or(Error::Other("Failed to parse VAT size"))?,
                    sp.next()
                        .ok_or(Error::Other("Failed to parse VAT validator time"))?,
                );
                let size = usize::from_str_radix(size, 16)
                    .map_err(|_| Error::Other("Failed to parse VAT size"))?;
                let time = u64::from_str_radix(time, 10)
                    .map_err(|_| Error::Other("Failed to parse VAT validity time"))?;
                build_page_cache(
                    builder.build_vat_cache(move |v, a| {
                        CachedVirtualTranslate::builder(v)
                            .arch(a)
                            .entries(size)
                            .validator(TimedCacheValidator::new(Duration::from_millis(time).into()))
                            .build()
                            .unwrap()
                    }),
                    mode,
                    args,
                )
            }
            None => build_page_cache(
                builder.build_vat_cache(|v, a| {
                    CachedVirtualTranslate::builder(v).arch(a).build().unwrap()
                }),
                mode,
                args,
            ),
        },
        None => build_page_cache(builder, mode, args),
    }
}

fn build_caches<
    A: 'static + PhysicalMemory + Clone,
    B: 'static + PhysicalMemory + Clone,
    C: 'static + VirtualTranslate + Clone,
>(
    builder: Win32KernelBuilder<A, B, C>,
    args: &Args,
) -> Result<OSInstance> {
    match args.get("memcache").unwrap_or("default") {
        "default" => build_kernel_hint(builder.build_default_caches(), args),
        "none" => build_kernel_hint(builder, args),
        mode => build_vat(builder, mode, args),
    }
}

fn build_dtb<
    A: 'static + PhysicalMemory + Clone,
    B: 'static + PhysicalMemory + Clone,
    C: 'static + VirtualTranslate + Clone,
>(
    builder: Win32KernelBuilder<A, B, C>,
    args: &Args,
) -> Result<OSInstance> {
    match args
        .get("dtb")
        .and_then(|d| u64::from_str_radix(d, 16).ok())
    {
        Some(dtb) => build_caches(builder.dtb(Address::from(dtb)), args),
        _ => build_caches(builder, args),
    }
}
