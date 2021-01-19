use super::{Args, OptionMut};
use crate::error::{Error, ToIntResult};
use crate::types::ReprCStr;
use std::mem::MaybeUninit;

pub extern "C" fn c_clone<T: Clone>(obj: &T) -> OptionMut<T> {
    let cloned_conn = Box::new(obj.clone());
    Some(Box::leak(cloned_conn))
}

pub unsafe extern "C" fn c_drop<T>(obj: &mut T) {
    let _: Box<T> = Box::from_raw(obj);
    // drop box
}

/// Wrapper for instantiating object with log level
///
/// This function will parse args into `Args`, log_level into `log::Level`,
/// and call the create_fn
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
        .int_out_result(out)
}

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
        .int_out_result(out)
}

/// Wrapper for instantiating object without logging
///
/// This function will parse args into `Args`, and call the create_fn
pub fn create_without_logging<T>(
    args: ReprCStr,
    out: &mut MaybeUninit<T>,
    create_fn: impl FnOnce(super::Args) -> Result<T, Error>,
) -> i32 {
    Args::parse(&args)
        .and_then(|args| create_fn(args))
        .int_out_result(out)
}
