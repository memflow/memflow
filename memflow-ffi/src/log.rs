use log::{debug, Level};
use memflow::cglue::IntError;
use memflow::error::Error;
use std::num::NonZeroI32;

#[no_mangle]
pub extern "C" fn log_init(level_num: i32) {
    let level = match level_num {
        0 => Level::Error,
        1 => Level::Warn,
        2 => Level::Info,
        3 => Level::Debug,
        4 => Level::Trace,
        _ => Level::Trace,
    };
    simple_logger::SimpleLogger::new()
        .with_level(level.to_level_filter())
        .init()
        .unwrap();
}

#[no_mangle]
pub extern "C" fn debug_error(error: i32) {
    if let Some(error) = NonZeroI32::new(error) {
        debug!("{}", <Error as IntError>::from_int_err(error));
    }
}
