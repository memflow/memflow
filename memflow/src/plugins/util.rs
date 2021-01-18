use super::{Args, OptionMut};
use crate::error::Error;
use log::error;
use std::ffi::CStr;
use std::mem::MaybeUninit;
use std::os::raw::c_char;

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
///
/// # Safety
///
/// args must be a valid null terminated string
pub unsafe fn create_with_logging<T>(
    args: *const c_char,
    log_level: i32,
    create_fn: impl Fn(&Args, log::Level) -> Result<T, Error>,
) -> std::option::Option<&'static mut T> {
    let level = match log_level {
        0 => ::log::Level::Error,
        1 => ::log::Level::Warn,
        2 => ::log::Level::Info,
        3 => ::log::Level::Debug,
        4 => ::log::Level::Trace,
        _ => ::log::Level::Trace,
    };

    let argsstr = CStr::from_ptr(args)
        .to_str()
        .map_err(|e| {
            ::log::error!("error converting connector args: {}", e);
            e
        })
        .ok()?;
    let conn_args = Args::parse(argsstr)
        .map_err(|e| {
            ::log::error!("error parsing connector args: {}", e);
            e
        })
        .ok()?;

    let conn = Box::new(
        create_fn(&conn_args, level)
            .map_err(|e| {
                ::log::error!("{}", e);
                e
            })
            .ok()?,
    );
    Some(Box::leak(conn))
}

/// Wrapper for instantiating object without logging
///
/// This function will parse args into `Args`, and call the create_fn
///
/// # Safety
///
/// args must be a valid null terminated string
pub unsafe fn create_without_logging<T>(
    args: *const c_char,
    create_fn: impl Fn(&super::Args) -> Result<T, Error>,
) -> std::option::Option<&'static mut T> {
    let argsstr = CStr::from_ptr(args).to_str().ok()?;
    let conn_args = Args::parse(argsstr).ok()?;

    let conn = Box::new(create_fn(&conn_args).ok()?);
    Some(Box::leak(conn))
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
