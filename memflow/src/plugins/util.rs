use super::{Args, OptionMut};
use crate::error::Error;
use crate::types::ReprCStr;
use log::error;
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
    create_fn: impl Fn(Args, log::Level) -> Result<T, Error>,
) -> i32 {
    let level = match log_level {
        0 => ::log::Level::Error,
        1 => ::log::Level::Warn,
        2 => ::log::Level::Info,
        3 => ::log::Level::Debug,
        4 => ::log::Level::Trace,
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

/// Wrapper for instantiating object without logging
///
/// This function will parse args into `Args`, and call the create_fn
pub fn create_without_logging<T>(
    args: ReprCStr,
    out: &mut MaybeUninit<T>,
    create_fn: impl Fn(super::Args) -> Result<T, Error>,
) -> i32 {
    Args::parse(&args)
        .and_then(|args| create_fn(args))
        .int_out_result(out)
}

pub trait ToIntResult<T> {
    fn int_result(self) -> i32;
    fn int_out_result(self, out: &mut MaybeUninit<T>) -> i32;

    fn int_result_logged(self) -> i32
    where
        Self: Sized,
    {
        let res = self.int_result();
        if res != 0 {
            error!("err value: {}", res);
        }
        res
    }
}

pub fn result_from_int_void(res: i32) -> Result<(), crate::error::Error> {
    if res == 0 {
        Ok(())
    } else {
        Err(Error::Other("C FFI Error"))
    }
}

pub fn part_result_from_int_void(res: i32) -> crate::error::PartialResult<()> {
    if res == 0 {
        Ok(())
    } else {
        Err(crate::error::PartialError::Error(
            crate::error::Error::Other("C FFI Error"),
        ))
    }
}

pub fn result_from_int<T>(res: i32, out: MaybeUninit<T>) -> Result<T, crate::error::Error> {
    if res == 0 {
        Ok(unsafe { out.assume_init() })
    } else {
        Err(Error::Other("C FFI Error"))
    }
}

impl<T, E: std::fmt::Display> ToIntResult<T> for Result<T, E> {
    fn int_result(self) -> i32 {
        if self.is_ok() {
            0
        } else {
            -1
        }
    }

    fn int_out_result(self, out: &mut MaybeUninit<T>) -> i32 {
        if let Ok(ret) = self {
            unsafe { out.as_mut_ptr().write(ret) };
            0
        } else {
            -1
        }
    }

    fn int_result_logged(self) -> i32 {
        if let Err(e) = self {
            error!("{}", e);
            -1
        } else {
            0
        }
    }
}
