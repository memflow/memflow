/*!
Specialized `Error` and `Result` types for memflow.
*/

use std::prelude::v1::*;
use std::{fmt, result, str};

use log::{debug, error, info, trace, warn};
use std::mem::MaybeUninit;

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

    pub const fn into_i32(self) -> i32 {
        let origin = ((self.0 as i32 + 1) & 0xFFFi32) << 4;
        let kind = ((self.1 as i32 + 1) & 0xFFFi32) << 16;
        -(1 + origin + kind)
    }

    pub fn from_i32(error: i32) -> Self {
        let origin = ((-error - 1) >> 4i32) & 0xFFFi32;
        let kind = ((-error - 1) >> 16i32) & 0xFFFi32;

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
    fn from(_err: PartialError<T>) -> Self {
        Error(ErrorOrigin::Memory, ErrorKind::PartialData)
    }
}

#[repr(u16)]
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum ErrorOrigin {
    Pointer,

    Args,
    ArgsValidator,

    Memory,
    MMU,
    MemoryMap,

    PhysicalMemory,
    VirtualTranslate,
    Cache,
    TLBCache,
    PageCache,
    VirtualMemory,

    Inventory,
    Connector,
    OSLayer,
    FFI,

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
            ErrorOrigin::MMU => "mmu",
            ErrorOrigin::MemoryMap => "memory map",

            ErrorOrigin::PhysicalMemory => "physical memory",
            ErrorOrigin::VirtualTranslate => "virtual translate",
            ErrorOrigin::Cache => "cache",
            ErrorOrigin::TLBCache => "tlb cache",
            ErrorOrigin::PageCache => "page cache",
            ErrorOrigin::VirtualMemory => "virtual memory",

            ErrorOrigin::Inventory => "inventory",
            ErrorOrigin::Connector => "connector",
            ErrorOrigin::OSLayer => "oslayer",
            ErrorOrigin::FFI => "ffi",

            ErrorOrigin::Other => "other",
        }
    }
}

#[repr(u16)]
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum ErrorKind {
    Uninitialized,
    NotSupported,
    NotImplemented,
    Configuration,
    Offset,
    HTTP,

    ArgNotExists,
    ArgValidation,
    RequiredArgNotFound,

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
    InvalidElfFile,
    InvalidPeFile,
    InvalidMachFile,
    MemflowExportsNotFound,
    VersionMismatch,
    AlreadyExists,
    PluginNotFound,
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
            ErrorKind::HTTP => "http error",

            ErrorKind::ArgNotExists => "the given argument does not exist",
            ErrorKind::ArgValidation => "the argument could not be validated",
            ErrorKind::RequiredArgNotFound => "required argument is not set",

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
            ErrorKind::InvalidElfFile => "file is not a valid elf file",
            ErrorKind::InvalidPeFile => "file is not a valid pe file",
            ErrorKind::InvalidMachFile => "file is not a valid mach file",
            ErrorKind::MemflowExportsNotFound => "file does not contain any memflow exports",
            ErrorKind::VersionMismatch => "version mismatch",
            ErrorKind::AlreadyExists => "already exists",
            ErrorKind::PluginNotFound => "plugin not found",
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
    /// Catch-all for all hard errors
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
    PartialVirtualWrite,
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
            PartialError::PartialVirtualWrite => "partial virtual write",
        }
    }

    /// Returns a static string representing the type of error.
    pub fn into_str(self) -> &'static str {
        self.as_str()
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

/// Specialized `PartialResult` exntesion for results.
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
            //Err(Error::PartialVirtualWrite(data)) => Ok(data),
            Err(_) => Err(Error(ErrorOrigin::Memory, ErrorKind::PartialData)),
        }
    }

    fn map_data<U, F: FnOnce(T) -> U>(self, func: F) -> PartialResult<U> {
        match self {
            Ok(data) => Ok(func(data)),
            Err(PartialError::Error(e)) => Err(PartialError::Error(e)),
            Err(PartialError::PartialVirtualRead(data)) => Ok(func(data)),
            Err(PartialError::PartialVirtualWrite) => Err(PartialError::PartialVirtualWrite),
        }
    }
}

pub trait AsIntResult<T> {
    fn into_int_result(self) -> i32;
    fn into_int_out_result(self, out: &mut MaybeUninit<T>) -> i32;
}

pub fn result_from_int_void(res: i32) -> Result<()> {
    if res == 0 {
        Ok(())
    } else {
        Err(Error::from_i32(res))
    }
}

pub fn result_from_int<T>(res: i32, out: MaybeUninit<T>) -> Result<T> {
    if res == 0 {
        Ok(unsafe { out.assume_init() })
    } else {
        Err(Error::from_i32(res))
    }
}

pub fn part_result_from_int_void(res: i32) -> PartialResult<()> {
    if res == 0 {
        Ok(())
    } else {
        let err = (-res) & 0xFi32;
        match err {
            1 => Err(PartialError::Error(Error::from_i32(res))),
            2 => Err(PartialError::PartialVirtualRead(())),
            3 => Err(PartialError::PartialVirtualWrite),
            _ => Err(PartialError::Error(Error(
                ErrorOrigin::FFI,
                ErrorKind::Unknown,
            ))),
        }
    }
}

pub fn part_result_from_int<T>(res: i32, out: MaybeUninit<T>) -> PartialResult<T> {
    if res == 0 {
        Ok(unsafe { out.assume_init() })
    } else {
        let err = (-res) & 0xFi32;
        match err {
            1 => Err(PartialError::Error(Error::from_i32(res))),
            2 => Err(PartialError::PartialVirtualRead(unsafe {
                out.assume_init()
            })),
            3 => Err(PartialError::PartialVirtualWrite),
            _ => Err(PartialError::Error(Error(
                ErrorOrigin::FFI,
                ErrorKind::Unknown,
            ))),
        }
    }
}

impl<T> AsIntResult<T> for result::Result<T, Error> {
    fn into_int_result(self) -> i32 {
        match self {
            Ok(_) => 0,
            Err(err) => err.into_i32(),
        }
    }

    fn into_int_out_result(self, out: &mut MaybeUninit<T>) -> i32 {
        match self {
            Ok(ret) => {
                unsafe { out.as_mut_ptr().write(ret) };
                0
            }
            Err(err) => err.into_i32(),
        }
    }
}

impl<T> AsIntResult<T> for result::Result<T, PartialError<T>> {
    fn into_int_result(self) -> i32 {
        match self {
            Ok(_) => 0,
            Err(PartialError::Error(err)) => err.into_i32(),
            Err(PartialError::PartialVirtualRead(_)) => -2,
            Err(PartialError::PartialVirtualWrite) => -3,
        }
    }

    fn into_int_out_result(self, out: &mut MaybeUninit<T>) -> i32 {
        match self {
            Ok(ret) => {
                unsafe { out.as_mut_ptr().write(ret) };
                0
            }
            Err(PartialError::Error(err)) => err.into_i32(),
            Err(PartialError::PartialVirtualRead(ret)) => {
                unsafe { out.as_mut_ptr().write(ret) };
                -2
            }
            Err(PartialError::PartialVirtualWrite) => -3,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        part_result_from_int, part_result_from_int_void, result_from_int, result_from_int_void,
        AsIntResult, Error, ErrorKind, ErrorOrigin, PartialError, PartialResult, Result,
    };
    use std::mem::MaybeUninit;

    #[test]
    pub fn error_from_i32_invalid() {
        let mut err = Error::from_i32(std::i32::MIN + 1);
        assert_eq!(err.0, ErrorOrigin::Other);
        assert_eq!(err.1, ErrorKind::Unknown);

        err = Error::from_i32(-1);
        assert_eq!(err.0, ErrorOrigin::Other);
        assert_eq!(err.1, ErrorKind::Unknown);

        err = Error::from_i32(-2);
        assert_eq!(err.0, ErrorOrigin::Other);
        assert_eq!(err.1, ErrorKind::Unknown);

        err = Error::from_i32(-3);
        assert_eq!(err.0, ErrorOrigin::Other);
        assert_eq!(err.1, ErrorKind::Unknown);
    }

    #[test]
    pub fn part_error_from_i32_invalid() {
        let mut result = part_result_from_int_void(-1);
        assert_eq!(result.is_ok(), false);
        assert_eq!(
            result.err().unwrap(),
            PartialError::Error(Error(ErrorOrigin::Other, ErrorKind::Unknown))
        );

        result = part_result_from_int_void(-2);
        assert_eq!(result.is_ok(), false);
        assert_eq!(result.err().unwrap(), PartialError::PartialVirtualRead(()));

        result = part_result_from_int_void(-3);
        assert_eq!(result.is_ok(), false);
        assert_eq!(result.err().unwrap(), PartialError::PartialVirtualWrite);

        result = part_result_from_int_void(-4);
        assert_eq!(result.is_ok(), false);
        assert_eq!(
            result.err().unwrap(),
            PartialError::Error(Error(ErrorOrigin::FFI, ErrorKind::Unknown))
        );
    }

    #[test]
    pub fn error_to_from_i32() {
        let err = Error::from_i32(Error(ErrorOrigin::Other, ErrorKind::InvalidPeFile).into_i32());
        assert_eq!(err.0, ErrorOrigin::Other);
        assert_eq!(err.1, ErrorKind::InvalidPeFile);
    }

    #[test]
    pub fn result_ok_void_ffi() {
        let r: Result<()> = Ok(());
        let result = result_from_int_void(r.into_int_result());
        assert_eq!(result.is_ok(), true);
    }

    #[test]
    pub fn result_ok_value_ffi() {
        let r: Result<i32> = Ok(1234i32);
        let mut out = MaybeUninit::<i32>::uninit();
        let result = result_from_int(r.into_int_out_result(&mut out), out);
        assert_eq!(result.is_ok(), true);
        assert_eq!(result.unwrap(), 1234i32);
    }

    #[test]
    pub fn result_error_void_ffi() {
        let r: Result<i32> = Err(Error(ErrorOrigin::Other, ErrorKind::InvalidPeFile));
        let result = result_from_int_void(r.into_int_result());
        assert_eq!(result.is_ok(), false);
        assert_eq!(result.err().unwrap().0, ErrorOrigin::Other);
        assert_eq!(result.err().unwrap().1, ErrorKind::InvalidPeFile);
    }

    #[test]
    pub fn result_error_value_ffi() {
        let r: Result<i32> = Err(Error(ErrorOrigin::Other, ErrorKind::InvalidPeFile));
        let mut out = MaybeUninit::<i32>::uninit();
        let result = result_from_int(r.into_int_out_result(&mut out), out);
        assert_eq!(result.is_ok(), false);
        assert_eq!(result.err().unwrap().0, ErrorOrigin::Other);
        assert_eq!(result.err().unwrap().1, ErrorKind::InvalidPeFile);
    }

    #[test]
    pub fn part_result_ok_void_ffi() {
        let r: PartialResult<()> = Ok(());
        let result = part_result_from_int_void(r.into_int_result());
        assert_eq!(result.is_ok(), true);
    }

    #[test]
    pub fn part_result_ok_value_ffi() {
        let r: PartialResult<i32> = Ok(1234i32);
        let mut out = MaybeUninit::<i32>::uninit();
        let result = part_result_from_int(r.into_int_out_result(&mut out), out);
        assert_eq!(result.is_ok(), true);
        assert_eq!(result.unwrap(), 1234i32);
    }

    #[test]
    pub fn part_result_error_void_ffi() {
        let r: PartialResult<i32> = Err(PartialError::Error(Error(
            ErrorOrigin::Other,
            ErrorKind::InvalidPeFile,
        )));
        let result = part_result_from_int_void(r.into_int_result());
        assert_eq!(result.is_ok(), false);
        assert_eq!(
            result.err().unwrap(),
            PartialError::Error(Error(ErrorOrigin::Other, ErrorKind::InvalidPeFile))
        );
    }

    #[test]
    pub fn part_result_error_value_ffi() {
        let r: PartialResult<i32> = Err(PartialError::Error(Error(
            ErrorOrigin::Other,
            ErrorKind::InvalidPeFile,
        )));
        let mut out = MaybeUninit::<i32>::uninit();
        let result = part_result_from_int(r.into_int_out_result(&mut out), out);
        assert_eq!(result.is_ok(), false);
        assert_eq!(
            result.err().unwrap(),
            PartialError::Error(Error(ErrorOrigin::Other, ErrorKind::InvalidPeFile))
        );
    }

    #[test]
    pub fn part_result_part_error_write_ffi() {
        let r: PartialResult<i32> = Err(PartialError::PartialVirtualWrite);
        let result = part_result_from_int_void(r.into_int_result());
        assert_eq!(result.is_ok(), false);
        assert_eq!(result.err().unwrap(), PartialError::PartialVirtualWrite);
    }

    #[test]
    pub fn part_result_part_error_read_ffi() {
        let r: PartialResult<i32> = Err(PartialError::PartialVirtualRead(1234i32));
        let mut out = MaybeUninit::<i32>::uninit();
        let result = part_result_from_int(r.into_int_out_result(&mut out), out);
        assert_eq!(result.is_ok(), false);
        assert_eq!(
            result.err().unwrap(),
            PartialError::PartialVirtualRead(1234i32)
        );
    }
}
