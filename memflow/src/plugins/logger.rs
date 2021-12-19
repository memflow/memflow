/// The plugin logger is just a thin wrapper which redirects all
/// logging functions from the callee to the caller
use crate::cglue::{
    ext::{DisplayBaseRef, DisplayRef},
    COption, CSliceRef, Opaquable,
};

use log::{Level, LevelFilter, SetLoggerError};

use core::ffi::c_void;

use std::sync::atomic::{AtomicPtr, Ordering};

/// FFI-Safe representation of log::Metadata
#[repr(C)]
pub struct Metadata<'a> {
    level: Level,
    target: CSliceRef<'a, u8>,
}

/// FFI-Safe representation of log::Record
#[repr(C)]
pub struct Record<'a> {
    metadata: Metadata<'a>,
    message: DisplayRef<'a>,
    module_path: COption<CSliceRef<'a, u8>>,
    file: COption<CSliceRef<'a, u8>>,
    line: COption<u32>,
    //#[cfg(feature = "kv_unstable")]
    //key_values: KeyValues<'a>,
}

/// A logger which just forwards all logging calls over the FFI
/// from the callee to the caller (i.e. from the plugin to the main process).
#[repr(C)]
pub struct PluginLogger {
    max_level: LevelFilter,
    enabled: extern "C" fn(metadata: &Metadata) -> bool,
    log: extern "C" fn(record: &Record) -> (),
    flush: extern "C" fn() -> (),
    on_level_change: AtomicPtr<c_void>,
}

impl PluginLogger {
    /// Creates a new PluginLogger.
    ///
    /// # Remarks:
    ///
    /// This function has to be called on the caller side
    /// (i.e. from memflow itself in the main process).
    pub fn new() -> Self {
        Self {
            max_level: log::max_level(),
            enabled: mf_log_enabled,
            log: mf_log_log,
            flush: mf_log_flush,
            on_level_change: AtomicPtr::new(std::ptr::null_mut()),
        }
    }

    /// Initializes the logger and sets up the logger in the log crate.
    ///
    /// # Remarks:
    ///
    /// This function has to be invoked on the callee side.
    /// (i.e. in the plugin)
    pub fn init(&'static self) -> Result<(), SetLoggerError> {
        // Explicitly typecheck the signature so that we do not mess anything up
        let val: SetMaxLevelFn = mf_log_set_max_level;
        self.on_level_change
            .store(val as *const c_void as *mut c_void, Ordering::SeqCst);
        log::set_max_level(self.max_level);
        log::set_logger(self)?;
        Ok(())
    }

    /// Updates the log level on the plugin end from local end
    pub fn on_level_change(&self, new_level: LevelFilter) {
        let val = self.on_level_change.load(Ordering::Relaxed);
        if let Some(on_change) = unsafe { std::mem::transmute::<_, Option<SetMaxLevelFn>>(val) } {
            on_change(new_level);
        }
    }
}

impl Default for PluginLogger {
    fn default() -> Self {
        PluginLogger::new()
    }
}

fn display_obj<'a, T: 'a + core::fmt::Display>(obj: &'a T) -> DisplayRef<'a> {
    let obj: DisplayBaseRef<T> = From::from(obj);
    obj.into_opaque()
}

impl log::Log for PluginLogger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        let m = Metadata {
            level: metadata.level(),
            target: metadata.target().into(),
        };
        (self.enabled)(&m)
    }

    fn log(&self, record: &log::Record) {
        let message = display_obj(record.args());
        let r = Record {
            metadata: Metadata {
                level: record.metadata().level(),
                target: record.metadata().target().into(),
            },
            message,
            module_path: record.module_path().map(|s| s.into()).into(),
            file: record.file().map(|s| s.into()).into(),
            line: record.line().into(),
        };
        (self.log)(&r)
    }

    fn flush(&self) {
        (self.flush)()
    }
}

type SetMaxLevelFn = extern "C" fn(LevelFilter);

/// FFI function which is being invoked from the main executable to the plugin library.
extern "C" fn mf_log_set_max_level(level: LevelFilter) {
    log::set_max_level(level);
}

/// FFI function which is being invoked from the plugin library to the main executable.
extern "C" fn mf_log_enabled(metadata: &Metadata) -> bool {
    log::logger().enabled(
        &log::Metadata::builder()
            .level(metadata.level)
            .target(unsafe { metadata.target.into_str() })
            .build(),
    )
}

/// FFI function which is being invoked from the plugin library to the main executable.
extern "C" fn mf_log_log(record: &Record) {
    log::logger().log(
        &log::Record::builder()
            .metadata(
                log::Metadata::builder()
                    .level(record.metadata.level)
                    .target(unsafe { record.metadata.target.into_str() })
                    .build(),
            )
            .args(format_args!("{}", record.message))
            .module_path(match &record.module_path {
                COption::Some(s) => Some(unsafe { s.into_str() }),
                COption::None => None,
            })
            .file(match &record.file {
                COption::Some(s) => Some(unsafe { s.into_str() }),
                COption::None => None,
            })
            .line(match &record.line {
                COption::Some(l) => Some(*l),
                COption::None => None,
            })
            .build(),
    )
}

/// FFI function which is being invoked from the plugin library to the main executable.
extern "C" fn mf_log_flush() {
    log::logger().flush()
}
