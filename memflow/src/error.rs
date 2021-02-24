/*!
Specialized `Error` and `Result` types for memflow.
*/

use std::prelude::v1::*;
use std::{fmt, result, str};

use log::{debug, error, info, warn};
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

    pub fn log_error(self, err: impl std::fmt::Display) -> Self {
        error!("{}/{}: {}", self.0.to_str(), self.1.to_str(), err);
        self
    }

    pub fn log_warn(self, err: impl std::fmt::Display) -> Self {
        warn!("{}/{}: {}", self.0.to_str(), self.1.to_str(), err);
        self
    }

    pub fn log_info(self, err: impl std::fmt::Display) -> Self {
        info!("{}/{}: {}", self.0.to_str(), self.1.to_str(), err);
        self
    }

    pub fn log_debug(self, err: impl std::fmt::Display) -> Self {
        debug!("{}/{}: {}", self.0.to_str(), self.1.to_str(), err);
        self
    }

    pub fn log_trace(self, err: impl std::fmt::Display) -> Self {
        debug!("{}/{}: {}", self.0.to_str(), self.1.to_str(), err);
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

#[repr(u8)]
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum ErrorOrigin {
    Pointer,

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

#[repr(u8)]
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum ErrorKind {
    Unknown,
    Uninitialized,
    NotSupported,
    NotImplemented,
    Configuration,
    Offset,
    HTTP,

    PartialData,

    EntryNotFound,
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
}

impl ErrorKind {
    /// Returns a static string representing the type of error.
    pub fn to_str(self) -> &'static str {
        match self {
            ErrorKind::Unknown => "unknown error",
            ErrorKind::Uninitialized => "unitialized",
            ErrorKind::NotSupported => "not supported",
            ErrorKind::NotImplemented => "not implemented",
            ErrorKind::Configuration => "configuration error",
            ErrorKind::Offset => "offset error",
            ErrorKind::HTTP => "http error",

            ErrorKind::PartialData => "partial data",

            ErrorKind::EntryNotFound => "entry not found",
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

// TODO: expose the exact error enum variant
const RES_INT_SUCCESS: i32 = 0;
const RES_INT_ERROR: i32 = -1;
const RES_INT_PARTIAL_READ_ERROR: i32 = -2;
const RES_INT_PARTIAL_WRITE_ERROR: i32 = -3;

pub trait AsIntResult<T> {
    fn as_int_result(self) -> i32;
    fn as_int_out_result(self, out: &mut MaybeUninit<T>) -> i32;

    fn as_int_result_logged(self) -> i32
    where
        Self: Sized,
    {
        let res = self.as_int_result();
        if res != RES_INT_SUCCESS {
            error!("err value: {}", res);
        }
        res
    }
}

pub fn result_from_int_void(res: i32) -> Result<()> {
    if res == RES_INT_SUCCESS {
        Ok(())
    } else {
        Err(Error(ErrorOrigin::FFI, ErrorKind::Unknown))
    }
}

pub fn result_from_int<T>(res: i32, out: MaybeUninit<T>) -> Result<T> {
    if res == RES_INT_SUCCESS {
        Ok(unsafe { out.assume_init() })
    } else {
        Err(Error(ErrorOrigin::FFI, ErrorKind::Unknown))
    }
}

pub fn part_result_from_int_void(res: i32) -> PartialResult<()> {
    match res {
        RES_INT_SUCCESS => Ok(()),
        RES_INT_ERROR => Err(PartialError::Error(Error(
            ErrorOrigin::FFI,
            ErrorKind::Unknown,
        ))),
        RES_INT_PARTIAL_READ_ERROR => Err(PartialError::PartialVirtualRead(())),
        RES_INT_PARTIAL_WRITE_ERROR => Err(PartialError::PartialVirtualWrite),
        _ => Err(PartialError::Error(Error(
            ErrorOrigin::FFI,
            ErrorKind::Unknown,
        ))),
    }
}

impl<T> AsIntResult<T> for result::Result<T, Error> {
    fn as_int_result(self) -> i32 {
        if self.is_ok() {
            RES_INT_SUCCESS
        } else {
            RES_INT_ERROR
        }
    }

    fn as_int_out_result(self, out: &mut MaybeUninit<T>) -> i32 {
        if let Ok(ret) = self {
            unsafe { out.as_mut_ptr().write(ret) };
            RES_INT_SUCCESS
        } else {
            RES_INT_ERROR
        }
    }

    fn as_int_result_logged(self) -> i32 {
        if let Err(e) = self {
            error!("{}", e);
            RES_INT_ERROR
        } else {
            RES_INT_SUCCESS
        }
    }
}

impl<T> AsIntResult<T> for result::Result<T, PartialError<T>> {
    fn as_int_result(self) -> i32 {
        match self {
            Ok(_) => RES_INT_SUCCESS,
            Err(PartialError::Error(_)) => RES_INT_ERROR,
            Err(PartialError::PartialVirtualRead(_)) => RES_INT_PARTIAL_READ_ERROR,
            Err(PartialError::PartialVirtualWrite) => RES_INT_PARTIAL_WRITE_ERROR,
        }
    }

    fn as_int_out_result(self, out: &mut MaybeUninit<T>) -> i32 {
        match self {
            Ok(ret) => {
                unsafe { out.as_mut_ptr().write(ret) };
                RES_INT_SUCCESS
            }
            Err(PartialError::Error(_)) => RES_INT_ERROR,
            Err(PartialError::PartialVirtualRead(ret)) => {
                unsafe { out.as_mut_ptr().write(ret) };
                RES_INT_PARTIAL_READ_ERROR
            }
            Err(PartialError::PartialVirtualWrite) => RES_INT_PARTIAL_WRITE_ERROR,
        }
    }

    fn as_int_result_logged(self) -> i32 {
        match self {
            Ok(_) => RES_INT_SUCCESS,
            Err(PartialError::Error(e)) => {
                error!("{}", e);
                RES_INT_ERROR
            }
            Err(PartialError::PartialVirtualRead(_)) => RES_INT_PARTIAL_READ_ERROR,
            Err(PartialError::PartialVirtualWrite) => RES_INT_PARTIAL_WRITE_ERROR,
        }
    }
}
