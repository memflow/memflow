use log::{Level, LevelFilter};
use memflow::cglue::IntError;
use memflow::error::Error;
use memflow::plugins::Inventory;
use std::num::NonZeroI32;

use std::ffi::CStr;
use std::os::raw::c_char;

/// Initialize logging with selected logging level.
#[no_mangle]
pub extern "C" fn log_init(level_filter: LevelFilter) {
    simplelog::TermLogger::init(
        level_filter,
        simplelog::Config::default(),
        simplelog::TerminalMode::Stdout,
        simplelog::ColorChoice::Auto,
    )
    .unwrap();
}

// TODO: add variadic functions when this is being stabilized, see https://github.com/rust-lang/rust/issues/44930

/// Logs a error message via log::error!
///
/// # Safety
///
/// The provided string must be a valid null-terminated char array.
#[no_mangle]
pub unsafe extern "C" fn log_error(s: *const c_char) {
    if !s.is_null() {
        let c_str = CStr::from_ptr(s);
        if let Ok(r_str) = c_str.to_str() {
            log::error!("{}", r_str);
        }
    }
}

/// Logs a warning message via log::warn!
///
/// # Safety
///
/// The provided string must be a valid null-terminated char array.
#[no_mangle]
pub unsafe extern "C" fn log_warn(s: *const c_char) {
    if !s.is_null() {
        let c_str = CStr::from_ptr(s);
        if let Ok(r_str) = c_str.to_str() {
            log::warn!("{}", r_str);
        }
    }
}

/// Logs a info message via log::info!
///
/// # Safety
///
/// The provided string must be a valid null-terminated char array.
#[no_mangle]
pub unsafe extern "C" fn log_info(s: *const c_char) {
    if !s.is_null() {
        let c_str = CStr::from_ptr(s);
        if let Ok(r_str) = c_str.to_str() {
            log::info!("{}", r_str);
        }
    }
}

/// Logs a debug message via log::debug!
///
/// # Safety
///
/// The provided string must be a valid null-terminated char array.
#[no_mangle]
pub unsafe extern "C" fn log_debug(s: *const c_char) {
    if !s.is_null() {
        let c_str = CStr::from_ptr(s);
        if let Ok(r_str) = c_str.to_str() {
            log::debug!("{}", r_str);
        }
    }
}

/// Logs a trace message via log::trace!
///
/// # Safety
///
/// The provided string must be a valid null-terminated char array.
#[no_mangle]
pub unsafe extern "C" fn log_trace(s: *const c_char) {
    if !s.is_null() {
        let c_str = CStr::from_ptr(s);
        if let Ok(r_str) = c_str.to_str() {
            log::trace!("{}", r_str);
        }
    }
}

/// Logs an error code with custom log level.
#[no_mangle]
pub extern "C" fn log_errorcode(level: Level, error: i32) {
    if let Some(error) = NonZeroI32::new(error) {
        log::log!(level, "{}", <Error as IntError>::from_int_err(error));
    }
}

/// Logs an error with debug log level.
#[no_mangle]
pub extern "C" fn log_debug_errorcode(error: i32) {
    log_errorcode(Level::Debug, error)
}

/// Sets new maximum log level.
///
/// If `inventory` is supplied, the log level is also updated within all plugin instances. However,
/// if it is not supplied, plugins will not have their log levels updated, potentially leading to
/// lower performance, or less logging than expected.
#[no_mangle]
pub extern "C" fn log_set_max_level(level_filter: LevelFilter, inventory: Option<&Inventory>) {
    if let Some(inventory) = inventory {
        inventory.set_max_log_level(level_filter);
    } else {
        log::set_max_level(level_filter);
    }
}
