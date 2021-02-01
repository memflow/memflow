use super::{Args, OptionMut};
use crate::error::{AsIntResult, Error};
use crate::types::ReprCStr;

use std::mem::MaybeUninit;
use std::path::Path;

use goblin::elf::Elf;

pub extern "C" fn c_clone<T: Clone>(obj: &T) -> OptionMut<T> {
    let cloned_conn = Box::new(obj.clone());
    Some(Box::leak(cloned_conn))
}

pub unsafe extern "C" fn c_drop<T>(obj: &mut T) {
    let _: Box<T> = Box::from_raw(obj);
    // drop box
}

#[cfg(target_os = "linux")]
pub fn find_export_by_prefix(
    path: impl AsRef<Path>,
    prefix: &str,
) -> crate::error::Result<Vec<String>> {
    let buffer =
        std::fs::read(path.as_ref()).map_err(|_| Error::Connector("file could not be read"))?;
    let elf = Elf::parse(buffer.as_slice())
        .map_err(|_| Error::Connector("file is not a valid elf file"))?;
    Ok(elf
        .syms
        .iter()
        .filter_map(|s| {
            if let Some(Ok(name)) = elf.strtab.get(s.st_name) {
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
pub fn find_export_by_prefix(path: impl AsRef<Path>) -> crate::error::Result<Vec<String>> {
    Err(Error::Connector(
        "find_export_by_prefix not implemented on windows yet",
    ))
}

#[cfg(target_os = "macos")]
pub fn find_export_by_prefix(path: impl AsRef<Path>) -> crate::error::Result<Vec<String>> {
    Err(Error::Connector(
        "find_export_by_prefix not implemented on mac yet",
    ))
}

/// Wrapper for instantiating object with log level
///
/// This function will parse args into `Args`, log_level into `log::Level`,
/// and call the create_fn
///
/// This function is used by the proc macros
pub fn create_with_logging<T>(
    args: ReprCStr,
    log_level: i32,
    out: &mut MaybeUninit<T>,
    create_fn: impl FnOnce(Args, log::Level) -> Result<T, Error>,
) -> i32 {
    let level = match log_level {
        1 => ::log::Level::Error,
        2 => ::log::Level::Warn,
        3 => ::log::Level::Info,
        4 => ::log::Level::Debug,
        5 => ::log::Level::Trace,
        _ => ::log::Level::Trace,
    };

    Args::parse(&args)
        .map_err(|e| {
            ::log::error!("error parsing args: {}", e);
            e
        })
        .and_then(|args| {
            create_fn(args, level).map_err(|e| {
                ::log::error!("{}", e);
                e
            })
        })
        .as_int_out_result(out)
}

/// Wrapper for instantiating object with all needed parameters
///
/// This function will parse args into `Args`, log_level into `log::Level`,
/// and call the create_fn with `input` forwarded.
///
/// This function is used by the proc macros
pub fn create_bare<T, I>(
    args: ReprCStr,
    input: I,
    log_level: i32,
    out: &mut MaybeUninit<T>,
    create_fn: impl FnOnce(&Args, I, log::Level) -> Result<T, Error>,
) -> i32 {
    let level = match log_level {
        1 => ::log::Level::Error,
        2 => ::log::Level::Warn,
        3 => ::log::Level::Info,
        4 => ::log::Level::Debug,
        5 => ::log::Level::Trace,
        _ => ::log::Level::Trace,
    };

    Args::parse(&args)
        .map_err(|e| {
            ::log::error!("error parsing args: {}", e);
            e
        })
        .and_then(|args| {
            create_fn(&args, input, level).map_err(|e| {
                ::log::error!("{}", e);
                e
            })
        })
        .as_int_out_result(out)
}

/// Wrapper for instantiating object without logging
///
/// This function will parse args into `Args`, and call the create_fn
///
/// This function is used by the proc macros
pub fn create_without_logging<T>(
    args: ReprCStr,
    out: &mut MaybeUninit<T>,
    create_fn: impl FnOnce(super::Args) -> Result<T, Error>,
) -> i32 {
    Args::parse(&args)
        .and_then(|args| create_fn(args))
        .as_int_out_result(out)
}
