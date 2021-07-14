use crate::cglue::{result::into_int_out_result, *};
use crate::error::{Error, ErrorKind, ErrorOrigin};

use super::Args;

use std::ffi::c_void;

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
    use goblin::mach::Mach;

    let buffer = std::fs::read(path.as_ref())
        .map_err(|err| Error(ErrorOrigin::Inventory, ErrorKind::UnableToReadFile).log_trace(err))?;
    let mach = Mach::parse(buffer.as_slice())
        .map_err(|err| Error(ErrorOrigin::Inventory, ErrorKind::InvalidExeFile).log_trace(err))?;
    let macho = match mach {
        Mach::Binary(mach) => mach,
        Mach::Fat(mach) => (0..mach.narches)
            .filter_map(|i| mach.get(i).ok())
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

/// Wrapper for instantiating object with log level
///
/// This function will parse args into `Args`, log_level into `log::Level`,
/// and call the create_fn
///
/// This function is used by the proc macros
pub fn create_with_logging<T>(
    args: &ReprCString,
    lib: COptArc<c_void>,
    log_level: i32,
    out: &mut MaybeUninit<T>,
    create_fn: impl FnOnce(Args, COptArc<c_void>, log::Level) -> Result<T, Error>,
) -> i32 {
    let level = match log_level {
        1 => ::log::Level::Error,
        2 => ::log::Level::Warn,
        3 => ::log::Level::Info,
        4 => ::log::Level::Debug,
        5 => ::log::Level::Trace,
        _ => ::log::Level::Trace,
    };

    into_int_out_result(
        Args::parse(&args)
            .map_err(|e| {
                ::log::error!("error parsing args: {}", e);
                e
            })
            .and_then(|args| {
                create_fn(args, lib, level).map_err(|e| {
                    ::log::error!("{}", e);
                    e
                })
            }),
        out,
    )
}

/// Wrapper for instantiating object with all needed parameters
///
/// This function will parse args into `Args`, log_level into `log::Level`,
/// and call the create_fn with `input` forwarded.
///
/// This function is used by the proc macros
pub fn create_bare<T, I>(
    args: &ReprCString,
    input: I,
    lib: COptArc<c_void>,
    log_level: i32,
    out: &mut MaybeUninit<T>,
    create_fn: impl FnOnce(&Args, I, COptArc<c_void>, log::Level) -> Result<T, Error>,
) -> i32 {
    let level = match log_level {
        1 => ::log::Level::Error,
        2 => ::log::Level::Warn,
        3 => ::log::Level::Info,
        4 => ::log::Level::Debug,
        5 => ::log::Level::Trace,
        _ => ::log::Level::Trace,
    };

    into_int_out_result(
        Args::parse(&args)
            .map_err(|e| {
                ::log::error!("error parsing args: {}", e);
                e
            })
            .and_then(|args| {
                create_fn(&args, input, lib, level).map_err(|e| {
                    ::log::error!("{}", e);
                    e
                })
            }),
        out,
    )
}

/// Wrapper for instantiating object without logging
///
/// This function will parse args into `Args`, and call the create_fn
///
/// This function is used by the proc macros
pub fn create_without_logging<T>(
    args: &ReprCString,
    lib: COptArc<c_void>,
    out: &mut MaybeUninit<T>,
    create_fn: impl FnOnce(super::Args, COptArc<c_void>) -> Result<T, Error>,
) -> i32 {
    into_int_out_result(
        Args::parse(&args).and_then(|args| create_fn(args, lib)),
        out,
    )
}
