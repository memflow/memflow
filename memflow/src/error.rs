/*!
Specialized `Error` and `Result` types for memflow.
*/

use std::num::NonZeroI32;
use std::prelude::v1::*;
use std::{fmt, result, str};

use log::{debug, error, info, trace, warn};

use crate::cglue::IntError;

#[cfg(feature = "std")]
use std::error;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct Error(pub ErrorOrigin, pub ErrorKind);

impl Error {
    /// Returns a static string representing the type of error.
    pub fn as_str(&self) -> &'static str {
        self.1.to_str()
    }

    /// Returns a static string representing the type of error.
    pub fn into_str(self) -> &'static str {
        self.as_str()
    }

    pub fn log_error(self, err: impl std::fmt::Display) -> Self {
        error!("{}: {} ({})", self.0.to_str(), self.1.to_str(), err);
        self
    }

    pub fn log_warn(self, err: impl std::fmt::Display) -> Self {
        warn!("{}: {} ({})", self.0.to_str(), self.1.to_str(), err);
        self
    }

    pub fn log_info(self, err: impl std::fmt::Display) -> Self {
        info!("{}: {} ({})", self.0.to_str(), self.1.to_str(), err);
        self
    }

    pub fn log_debug(self, err: impl std::fmt::Display) -> Self {
        debug!("{}: {} ({})", self.0.to_str(), self.1.to_str(), err);
        self
    }

    pub fn log_trace(self, err: impl std::fmt::Display) -> Self {
        trace!("{}: {} ({})", self.0.to_str(), self.1.to_str(), err);
        self
    }
}

impl IntError for Error {
    fn into_int_err(self) -> NonZeroI32 {
        let origin = ((self.0 as i32 + 1) & 0xFFFi32) << 4;
        let kind = ((self.1 as i32 + 1) & 0xFFFi32) << 16;
        NonZeroI32::new(-(1 + origin + kind)).unwrap()
    }

    fn from_int_err(err: NonZeroI32) -> Self {
        let origin = ((-err.get() - 1) >> 4i32) & 0xFFFi32;
        let kind = ((-err.get() - 1) >> 16i32) & 0xFFFi32;

        let error_origin = if origin > 0 && origin <= ErrorOrigin::Other as i32 + 1 {
            unsafe { std::mem::transmute(origin as u16 - 1) }
        } else {
            ErrorOrigin::Other
        };

        let error_kind = if kind > 0 && kind <= ErrorKind::Unknown as i32 + 1 {
            unsafe { std::mem::transmute(kind as u16 - 1) }
        } else {
            ErrorKind::Unknown
        };

        Self(error_origin, error_kind)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}: {}", self.0.to_str(), self.1.to_str())
    }
}

#[cfg(feature = "std")]
impl error::Error for Error {
    fn description(&self) -> &str {
        self.as_str()
    }
}

/// Convert from PartialError
impl<T> From<PartialError<T>> for Error {
    fn from(err: PartialError<T>) -> Self {
        match err {
            PartialError::Error(e) => e,
            _ => Error(ErrorOrigin::Memory, ErrorKind::PartialData),
        }
    }
}

impl From<ErrorOrigin> for Error {
    fn from(origin: ErrorOrigin) -> Self {
        Error(origin, ErrorKind::Unknown)
    }
}

impl From<ErrorKind> for Error {
    fn from(kind: ErrorKind) -> Self {
        Error(ErrorOrigin::Other, kind)
    }
}

#[repr(u16)]
#[non_exhaustive]
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum ErrorOrigin {
    Pointer,

    Args,
    ArgsValidator,

    Memory,
    Mmu,
    MemoryMap,

    PhysicalMemory,
    VirtualTranslate,
    Cache,
    TlbCache,
    PageCache,
    VirtualMemory,

    Inventory,
    Connector,
    OsLayer,
    Ffi,

    Other,
}

impl ErrorOrigin {
    /// Returns a static string representing the type of error.
    pub fn to_str(self) -> &'static str {
        match self {
            ErrorOrigin::Pointer => "pointer",

            ErrorOrigin::Args => "args",
            ErrorOrigin::ArgsValidator => "args validator",

            ErrorOrigin::Memory => "memory",
            ErrorOrigin::Mmu => "mmu",
            ErrorOrigin::MemoryMap => "memory map",

            ErrorOrigin::PhysicalMemory => "physical memory",
            ErrorOrigin::VirtualTranslate => "virtual translate",
            ErrorOrigin::Cache => "cache",
            ErrorOrigin::TlbCache => "tlb cache",
            ErrorOrigin::PageCache => "page cache",
            ErrorOrigin::VirtualMemory => "virtual memory",

            ErrorOrigin::Inventory => "inventory",
            ErrorOrigin::Connector => "connector",
            ErrorOrigin::OsLayer => "oslayer",
            ErrorOrigin::Ffi => "ffi",

            ErrorOrigin::Other => "other",
        }
    }
}

#[repr(u16)]
#[non_exhaustive]
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum ErrorKind {
    Uninitialized,
    NotSupported,
    NotImplemented,
    Configuration,
    Offset,
    Http,

    ArgNotExists,
    ArgValidation,
    RequiredArgNotFound,

    InvalidArgument,

    PartialData,

    NotFound,
    OutOfBounds,
    OutOfMemoryRange,
    Encoding,

    InvalidPath,
    ReadOnly,
    UnableToReadDir,
    UnableToReadDirEntry,
    UnableToReadFile,
    UnableToCreateDirectory,
    UnableToWriteFile,
    UnableToSeekFile,

    UnableToMapFile,
    MemoryMapOutOfRange,
    UnableToReadMemory,

    InvalidArchitecture,
    InvalidMemorySize,
    InvalidMemorySizeUnit,

    UnableToLoadLibrary,
    InvalidExeFile,
    MemflowExportsNotFound,
    VersionMismatch,
    AlreadyExists,
    PluginNotFound,
    TargetNotFound,
    InvalidAbi,
    UnsupportedOptionalFeature,

    ProcessNotFound,
    InvalidProcessInfo,
    ModuleNotFound,
    ExportNotFound,
    ImportNotFound,
    SectionNotFound,

    Unknown,
}

impl ErrorKind {
    /// Returns a static string representing the type of error.
    pub fn to_str(self) -> &'static str {
        match self {
            ErrorKind::Uninitialized => "unitialized",
            ErrorKind::NotSupported => "not supported",
            ErrorKind::NotImplemented => "not implemented",
            ErrorKind::Configuration => "configuration error",
            ErrorKind::Offset => "offset error",
            ErrorKind::Http => "http error",

            ErrorKind::ArgNotExists => "the given argument does not exist",
            ErrorKind::ArgValidation => "the argument could not be validated",
            ErrorKind::RequiredArgNotFound => "required argument is not set",

            ErrorKind::InvalidArgument => "invalid argument passed",

            ErrorKind::PartialData => "partial data",

            ErrorKind::NotFound => "not found",
            ErrorKind::OutOfBounds => "out of bounds",
            ErrorKind::OutOfMemoryRange => "out of memory range",
            ErrorKind::Encoding => "encoding error",

            ErrorKind::InvalidPath => "invalid path",
            ErrorKind::ReadOnly => "trying to write to a read only resource",
            ErrorKind::UnableToReadDir => "unable to read directory",
            ErrorKind::UnableToReadDirEntry => "unable to read directory entry",
            ErrorKind::UnableToReadFile => "unable to read file",
            ErrorKind::UnableToCreateDirectory => "unable to create directory",
            ErrorKind::UnableToWriteFile => "unable to write file",
            ErrorKind::UnableToSeekFile => "unable to seek file",

            ErrorKind::UnableToMapFile => "unable to map file",
            ErrorKind::MemoryMapOutOfRange => "memory map is out of range",
            ErrorKind::UnableToReadMemory => "unable to read memory",

            ErrorKind::InvalidArchitecture => "invalid architecture",
            ErrorKind::InvalidMemorySize => "invalid memory size",
            ErrorKind::InvalidMemorySizeUnit => "invalid memory size units (or none)",

            ErrorKind::UnableToLoadLibrary => "unable to load library",
            ErrorKind::InvalidExeFile => "file is not a valid executable file",
            ErrorKind::MemflowExportsNotFound => "file does not contain any memflow exports",
            ErrorKind::VersionMismatch => "version mismatch",
            ErrorKind::AlreadyExists => "already exists",
            ErrorKind::PluginNotFound => "plugin not found",
            ErrorKind::TargetNotFound => "specified (connector) target could not be found",
            ErrorKind::InvalidAbi => "invalid plugin ABI",
            ErrorKind::UnsupportedOptionalFeature => "unsupported optional feature",

            ErrorKind::ProcessNotFound => "process not found",
            ErrorKind::InvalidProcessInfo => "invalid process info",
            ErrorKind::ModuleNotFound => "module not found",
            ErrorKind::ExportNotFound => "export not found",
            ErrorKind::ImportNotFound => "import not found",
            ErrorKind::SectionNotFound => "section not found",

            ErrorKind::Unknown => "unknown error",
        }
    }
}

/// Specialized `PartialError` type for recoverable memflow errors.
#[derive(Clone, Eq, PartialEq, Hash)]
pub enum PartialError<T> {
    /// Hard Error
    ///
    /// Catch-all for all hard  
    Error(Error),
    /// Partial Virtual Read Error
    ///
    /// Error when a read from virtual memory only completed partially.
    /// This can usually happen when trying to read a page that is currently paged out.
    PartialVirtualRead(T),
    /// Partial Virtual Write Error
    ///
    /// Error when a write from virtual memory only completed partially.
    /// This can usually happen when trying to read a page that is currently paged out.
    PartialVirtualWrite(T),
}

/// Convert from Error
impl<T> From<Error> for PartialError<T> {
    fn from(err: Error) -> Self {
        PartialError::Error(err)
    }
}

impl<T> PartialError<T> {
    /// Returns a static string representing the type of error.
    pub fn as_str(&self) -> &'static str {
        match self {
            PartialError::Error(e) => e.as_str(),
            PartialError::PartialVirtualRead(_) => "partial virtual read",
            PartialError::PartialVirtualWrite(_) => "partial virtual write",
        }
    }

    /// Returns a static string representing the type of error.
    pub fn into_str(self) -> &'static str {
        self.as_str()
    }
}

impl IntError for PartialError<()> {
    fn into_int_err(self) -> NonZeroI32 {
        match self {
            PartialError::Error(err) => err.into_int_err(),
            PartialError::PartialVirtualRead(_) => NonZeroI32::new(-2).unwrap(),
            PartialError::PartialVirtualWrite(_) => NonZeroI32::new(-3).unwrap(),
        }
    }

    fn from_int_err(err: NonZeroI32) -> Self {
        let errc = (-err.get()) & 0xFi32;
        match errc {
            1 => PartialError::Error(Error::from_int_err(err)),
            2 => PartialError::PartialVirtualRead(()),
            3 => PartialError::PartialVirtualWrite(()),
            _ => PartialError::Error(Error(ErrorOrigin::Ffi, ErrorKind::Unknown)),
        }
    }
}

/// Custom fmt::Debug impl for the specialized memflow `Error` type.
/// This is required due to our generic type T.
impl<T> fmt::Debug for PartialError<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.to_string())
    }
}

impl<T> fmt::Display for PartialError<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            PartialError::Error(e) => f.write_str(e.as_str()),
            _ => f.write_str(self.as_str()),
        }
    }
}

#[cfg(feature = "std")]
impl<T: fmt::Debug> error::Error for PartialError<T> {
    fn description(&self) -> &str {
        self.as_str()
    }
}

/// Specialized `Result` type for memflow results.
pub type Result<T> = result::Result<T, Error>;

/// Specialized `PartialResult` type for memflow results with recoverable errors.
pub type PartialResult<T> = result::Result<T, PartialError<T>>;

/// Specialized `PartialResult` extension for results.
pub trait PartialResultExt<T> {
    /// Tries to extract the data from the `Result`.
    /// This will return a full error even if a partial error happened.
    fn data(self) -> Result<T>;

    /// Tries to extract the data or partial data from the `Result`.
    /// This will return a full error only if a hard error happened.
    /// A partial error will be converted to an `Ok(T)`.
    fn data_part(self) -> Result<T>;

    /// Maps the data contained in the partial result to another result.
    /// This is especially useful if you want to return a different result type
    /// but want to keep the partial result information.
    fn map_data<U, F: FnOnce(T) -> U>(self, func: F) -> PartialResult<U>;
}

impl<T> PartialResultExt<T> for PartialResult<T> {
    fn data(self) -> Result<T> {
        match self {
            Ok(data) => Ok(data),
            Err(_) => Err(Error(ErrorOrigin::Memory, ErrorKind::PartialData)),
        }
    }

    fn data_part(self) -> Result<T> {
        match self {
            Ok(data) => Ok(data),
            Err(PartialError::PartialVirtualRead(data)) => Ok(data),
            Err(PartialError::PartialVirtualWrite(data)) => Ok(data),
            Err(PartialError::Error(e)) => Err(e),
        }
    }

    fn map_data<U, F: FnOnce(T) -> U>(self, func: F) -> PartialResult<U> {
        match self {
            Ok(data) => Ok(func(data)),
            Err(PartialError::Error(e)) => Err(PartialError::Error(e)),
            Err(PartialError::PartialVirtualRead(data)) => Ok(func(data)),
            Err(PartialError::PartialVirtualWrite(data)) => Ok(func(data)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cglue::result::{
        from_int_result, from_int_result_empty, into_int_out_result, into_int_result, IntError,
    };
    use std::mem::MaybeUninit;
    use std::num::NonZeroI32;

    #[test]
    pub fn error_from_i32_invalid() {
        let mut err = Error::from_int_err(NonZeroI32::new(std::i32::MIN + 1).unwrap());
        assert_eq!(err.0, ErrorOrigin::Other);
        assert_eq!(err.1, ErrorKind::Unknown);

        err = Error::from_int_err(NonZeroI32::new(-1).unwrap());
        assert_eq!(err.0, ErrorOrigin::Other);
        assert_eq!(err.1, ErrorKind::Unknown);

        err = Error::from_int_err(NonZeroI32::new(-2).unwrap());
        assert_eq!(err.0, ErrorOrigin::Other);
        assert_eq!(err.1, ErrorKind::Unknown);

        err = Error::from_int_err(NonZeroI32::new(-3).unwrap());
        assert_eq!(err.0, ErrorOrigin::Other);
        assert_eq!(err.1, ErrorKind::Unknown);
    }

    #[test]
    pub fn part_error_from_i32_invalid() {
        let mut result: PartialResult<()> = from_int_result_empty(-1);
        assert!(result.is_err());
        assert_eq!(
            result.err().unwrap(),
            PartialError::Error(Error(ErrorOrigin::Other, ErrorKind::Unknown))
        );

        result = from_int_result_empty(-2);
        assert!(result.is_err());
        assert_eq!(result.err().unwrap(), PartialError::PartialVirtualRead(()));

        result = from_int_result_empty(-3);
        assert!(result.is_err());
        assert_eq!(result.err().unwrap(), PartialError::PartialVirtualWrite(()));

        result = from_int_result_empty(-4);
        assert!(result.is_err());
        assert_eq!(
            result.err().unwrap(),
            PartialError::Error(Error(ErrorOrigin::Ffi, ErrorKind::Unknown))
        );
    }

    #[test]
    pub fn error_to_from_i32() {
        let err = Error::from_int_err(
            Error(ErrorOrigin::Other, ErrorKind::InvalidExeFile).into_int_err(),
        );
        assert_eq!(err.0, ErrorOrigin::Other);
        assert_eq!(err.1, ErrorKind::InvalidExeFile);
    }

    #[test]
    pub fn result_ok_void_ffi() {
        let r: Result<()> = Ok(());
        let result: Result<()> = from_int_result_empty(into_int_result(r));
        assert!(result.is_ok());
    }

    #[test]
    pub fn result_ok_value_ffi() {
        let r: Result<i32> = Ok(1234i32);
        let mut out = MaybeUninit::<i32>::uninit();
        let result: Result<i32> = unsafe { from_int_result(into_int_out_result(r, &mut out), out) };
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 1234i32);
    }

    #[test]
    pub fn result_error_void_ffi() {
        let r: Result<i32> = Err(Error(ErrorOrigin::Other, ErrorKind::InvalidExeFile));
        let result: Result<()> = from_int_result_empty(into_int_result(r));
        assert!(result.is_err());
        assert_eq!(result.err().unwrap().0, ErrorOrigin::Other);
        assert_eq!(result.err().unwrap().1, ErrorKind::InvalidExeFile);
    }

    #[test]
    pub fn result_error_value_ffi() {
        let r: Result<i32> = Err(Error(ErrorOrigin::Other, ErrorKind::InvalidExeFile));
        let mut out = MaybeUninit::<i32>::uninit();
        let result: Result<i32> = unsafe { from_int_result(into_int_out_result(r, &mut out), out) };
        assert!(result.is_err());
        assert_eq!(result.err().unwrap().0, ErrorOrigin::Other);
        assert_eq!(result.err().unwrap().1, ErrorKind::InvalidExeFile);
    }

    #[test]
    pub fn part_result_ok_void_ffi() {
        let r: PartialResult<()> = Ok(());
        let result: PartialResult<()> = from_int_result_empty(into_int_result(r));
        assert!(result.is_ok());
    }

    #[test]
    pub fn part_result_error_void_ffi() {
        let r: PartialResult<()> = Err(PartialError::Error(Error(
            ErrorOrigin::Other,
            ErrorKind::InvalidExeFile,
        )));
        let result: PartialResult<()> = from_int_result_empty(into_int_result(r));
        assert!(result.is_err());
        assert_eq!(
            result.err().unwrap(),
            PartialError::Error(Error(ErrorOrigin::Other, ErrorKind::InvalidExeFile))
        );
    }

    #[test]
    pub fn part_result_part_error_read_ffi() {
        let r: PartialResult<()> = Err(PartialError::PartialVirtualRead(()));
        let result: PartialResult<()> = from_int_result_empty(into_int_result(r));
        assert!(result.is_err());
        assert_eq!(result.err().unwrap(), PartialError::PartialVirtualRead(()));
    }

    #[test]
    pub fn part_result_part_error_write_ffi() {
        let r: PartialResult<()> = Err(PartialError::PartialVirtualWrite(()));
        let result: PartialResult<()> = from_int_result_empty(into_int_result(r));
        assert!(result.is_err());
        assert_eq!(result.err().unwrap(), PartialError::PartialVirtualWrite(()));
    }
}
