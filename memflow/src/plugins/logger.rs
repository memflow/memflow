/// The plugin logger is just a thin wrapper which redirects all
/// logging functions from the callee to the caller
use crate::cglue::{COption, ReprCString};

use log::{Level, SetLoggerError};

/// FFI-Safe representation of log::Metadata
#[repr(C)]
pub struct Metadata {
    level: i32,
    target: ReprCString,
}

/// FFI-Safe representation of log::Record
#[repr(C)]
pub struct Record {
    metadata: Metadata,
    message: ReprCString,
    module_path: COption<ReprCString>,
    file: COption<ReprCString>,
    line: COption<u32>,
    //#[cfg(feature = "kv_unstable")]
    //key_values: KeyValues<'a>,
}

/// A logger which just forwards all calls over the ffi from the callee to the caller.
#[repr(C)]
pub struct PluginLogger {
    max_level: i32,
    enabled: extern "C" fn(metadata: &Metadata) -> i32,
    log: extern "C" fn(record: &Record) -> (),
    flush: extern "C" fn() -> (),
}

impl PluginLogger {
    /// Creates a new PluginLogger. This function has to be called on the caller side.
    pub fn new() -> Self {
        Self {
            max_level: log::max_level() as i32,
            enabled: mf_log_enabled,
            log: mf_log_log,
            flush: mf_log_flush,
        }
    }

    /// Actually initializes the logger and sets up the log crate.
    /// This function has to be invoked on the callee side.
    pub fn init(self) -> Result<(), SetLoggerError> {
        log::set_max_level(i32_to_level(self.max_level).to_level_filter());
        // TODO: move to https://docs.rs/log/latest/log/fn.set_logger.html
        log::set_boxed_logger(Box::new(self))?;
        Ok(())
    }
}

impl Default for PluginLogger {
    fn default() -> Self {
        PluginLogger::new()
    }
}

impl log::Log for PluginLogger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        let m = Metadata {
            level: metadata.level() as i32,
            target: metadata.target().into(),
        };
        (self.enabled)(&m) != 0
    }

    fn log(&self, record: &log::Record) {
        let r = Record {
            metadata: Metadata {
                level: record.metadata().level() as i32,
                target: record.metadata().target().into(),
            },
            message: format!("{}", record.args()).into(),
            module_path: record.module_path().map(|s| s.into()).into(),
            file: record.file().map(|s| s.into()).into(),
            line: record.line().map(|s| s.into()).into(),
        };
        (self.log)(&r)
    }

    fn flush(&self) {
        (self.flush)()
    }
}

extern "C" fn mf_log_enabled(metadata: &Metadata) -> i32 {
    match log::logger().enabled(
        &log::Metadata::builder()
            .level(i32_to_level(metadata.level))
            .target(metadata.target.as_ref())
            .build(),
    ) {
        true => 1,
        false => 0,
    }
}

extern "C" fn mf_log_log(record: &Record) {
    log::logger().log(
        &log::Record::builder()
            .metadata(
                log::Metadata::builder()
                    .level(i32_to_level(record.metadata.level))
                    .target(record.metadata.target.as_ref())
                    .build(),
            )
            .args(format_args!("{}", record.message.as_ref()))
            .module_path(match &record.module_path {
                COption::Some(s) => Some(s.as_ref()),
                COption::None => None,
            })
            .file(match &record.file {
                COption::Some(s) => Some(s.as_ref()),
                COption::None => None,
            })
            .line(match &record.line {
                COption::Some(l) => Some(*l),
                COption::None => None,
            })
            .build(),
    )
}

extern "C" fn mf_log_flush() {
    log::logger().flush()
}

// internal helper functions to convert from i32 to level
#[inline]
fn i32_to_level(level: i32) -> Level {
    match level {
        1 => Level::Error,
        2 => Level::Warn,
        3 => Level::Info,
        4 => Level::Debug,
        5 => Level::Trace,
        _ => Level::Trace,
    }
}
