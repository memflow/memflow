use crate::cglue::result::into_int_out_result;
use crate::error::{Error, ErrorKind, ErrorOrigin};

use super::{LibArc, PluginLogger};

use std::mem::MaybeUninit;
use std::path::Path;

#[cfg(not(any(target_os = "windows", target_os = "macos")))]
pub fn find_export_by_prefix(
    path: impl AsRef<Path>,
    prefix: &str,
) -> crate::error::Result<Vec<String>> {
    use goblin::elf::Elf;

    let buffer = std::fs::read(path.as_ref())
        .map_err(|err| Error(ErrorOrigin::Inventory, ErrorKind::UnableToReadFile).log_trace(err))?;
    let elf = Elf::parse(buffer.as_slice())
        .map_err(|err| Error(ErrorOrigin::Inventory, ErrorKind::InvalidExeFile).log_trace(err))?;
    Ok(elf
        .syms
        .iter()
        .filter_map(|s| {
            if let Some(name) = elf.strtab.get_at(s.st_name) {
                match name.starts_with(prefix) {
                    true => Some(name.to_owned()),
                    false => None,
                }
            } else {
                None
            }
        })
        .collect::<Vec<_>>())
}

#[cfg(target_os = "windows")]
pub fn find_export_by_prefix(
    path: impl AsRef<Path>,
    prefix: &str,
) -> crate::error::Result<Vec<String>> {
    use goblin::pe::PE;

    let buffer = std::fs::read(path.as_ref())
        .map_err(|err| Error(ErrorOrigin::Inventory, ErrorKind::UnableToReadFile).log_trace(err))?;
    let pe = PE::parse(buffer.as_slice())
        .map_err(|err| Error(ErrorOrigin::Inventory, ErrorKind::InvalidExeFile).log_trace(err))?;
    Ok(pe
        .exports
        .iter()
        .filter_map(|s| s.name)
        .filter_map(|name| {
            if name.starts_with(prefix) {
                Some(name.to_owned())
            } else {
                None
            }
        })
        .collect::<Vec<_>>())
}

#[cfg(target_os = "macos")]
pub fn find_export_by_prefix(
    path: impl AsRef<Path>,
    prefix: &str,
) -> crate::error::Result<Vec<String>> {
    use goblin::mach::{Mach, SingleArch};

    let buffer = std::fs::read(path.as_ref())
        .map_err(|err| Error(ErrorOrigin::Inventory, ErrorKind::UnableToReadFile).log_trace(err))?;
    let mach = Mach::parse(buffer.as_slice())
        .map_err(|err| Error(ErrorOrigin::Inventory, ErrorKind::InvalidExeFile).log_trace(err))?;
    let macho = match mach {
        Mach::Binary(mach) => mach,
        Mach::Fat(mach) => (0..mach.narches)
            .filter_map(|i| mach.get(i).ok())
            .filter_map(|a| match a {
                SingleArch::MachO(mach) => Some(mach),
                SingleArch::Archive(_) => None,
            })
            .next()
            .ok_or_else(|| {
                Error(ErrorOrigin::Inventory, ErrorKind::InvalidExeFile)
                    .log_trace("failed to find valid MachO header!")
            })?,
    };

    // macho symbols are prefixed with `_` in the object file.
    let macho_prefix = "_".to_owned() + prefix;
    Ok(macho
        .symbols
        .ok_or_else(|| {
            Error(ErrorOrigin::Inventory, ErrorKind::InvalidExeFile)
                .log_trace("failed to parse MachO symbols!")
        })?
        .iter()
        .filter_map(|s| s.ok())
        .filter_map(|(name, _)| {
            // symbols should only contain ascii characters
            if name.starts_with(&macho_prefix) {
                Some(name[1..].to_owned())
            } else {
                None
            }
        })
        .collect::<Vec<_>>())
}

/// Wrapper for instantiating object.
///
/// This function will initialize the [`PluginLogger`],
/// parse args into `Args`, and call the create_fn
///
/// This function is used by the connector proc macro
pub fn wrap<A: Default, T>(
    args: Option<&A>,
    lib: LibArc,
    logger: Option<&'static PluginLogger>,
    out: &mut MaybeUninit<T>,
    create_fn: impl FnOnce(&A, LibArc) -> Result<T, Error>,
) -> i32 {
    if let Some(logger) = logger {
        logger.init().ok();
    }

    let default: A = Default::default();
    let args = args.unwrap_or(&default);

    into_int_out_result(create_fn(args, lib), out)
}

/// Wrapper for instantiating object with all needed parameters
///
/// This function will initialize the [`PluginLogger`],
/// parse args into `Args` and call the create_fn with `input` forwarded.
///
/// This function is used by the connector proc macro
pub fn wrap_with_input<A: Default, T, I>(
    args: Option<&A>,
    input: I,
    lib: LibArc,
    logger: Option<&'static PluginLogger>,
    out: &mut MaybeUninit<T>,
    create_fn: impl FnOnce(&A, I, LibArc) -> Result<T, Error>,
) -> i32 {
    if let Some(logger) = logger {
        logger.init().ok();
    }

    let default: A = Default::default();
    let args = args.unwrap_or(&default);

    into_int_out_result(
        create_fn(args, input, lib).map_err(|e| {
            ::log::error!("{}", e);
            e
        }),
        out,
    )
}
