/*!
Specialized `Error` and `Result` types for memflow.
*/

use std::prelude::v1::*;
use std::{convert, fmt, result, str};

#[cfg(feature = "std")]
use std::error;

/// Specialized `Error` type for memflow errors.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum Error {
    /// Generic error type containing a string
    Other(&'static str),
    /// Partial error.
    ///
    /// Catch-all for partial errors which have been
    /// converted into full errors.
    Partial,
    /// Out of bounds.
    ///
    /// Catch-all for bounds check errors.
    Bounds,
    /// IO error
    ///
    /// Catch-all for io related errors.
    IO(&'static str),
    /// Invalid Architecture error.
    ///
    /// The architecture provided is not a valid argument for the given function.
    InvalidArchitecture,
    /// Connector error
    ///
    /// Catch-all for connector related errors
    Connector(&'static str),
    /// Physical Read Error
    ///
    /// A read/write from/to the physical memory has failed.
    PhysicalMemory(&'static str),
    /// VirtualTranslate Error
    ///
    /// Error when trying to translate virtual to physical memory addresses.
    VirtualTranslate,
    /// Virtual Memory Error
    ///
    /// A read/write from/to the virtual memory has failed.
    VirtualMemory(&'static str),
    /// Encoding error.
    ///
    /// Catch-all for string related errors such as lacking a nul terminator.
    Encoding,
}

/// Convert from &str to error
impl convert::From<&'static str> for Error {
    fn from(error: &'static str) -> Self {
        Error::Other(error)
    }
}

/// Convert from str::Utf8Error
impl From<str::Utf8Error> for Error {
    fn from(_err: str::Utf8Error) -> Self {
        Error::Encoding
    }
}

/// Convert from PartialError
impl<T> From<PartialError<T>> for Error {
    fn from(_err: PartialError<T>) -> Self {
        Error::Partial
    }
}

impl Error {
    /// Returns a tuple representing the error description and its string value.
    pub fn to_str_pair(self) -> (&'static str, Option<&'static str>) {
        match self {
            Error::Other(e) => ("other error", Some(e)),
            Error::Partial => ("partial error", None),
            Error::Bounds => ("out of bounds", None),
            Error::IO(e) => ("io error", Some(e)),
            Error::InvalidArchitecture => ("invalid architecture", None),
            Error::Connector(e) => ("connector error", Some(e)),
            Error::PhysicalMemory(e) => ("physical memory error", Some(e)),
            Error::VirtualTranslate => ("virtual address translation failed", None),
            Error::VirtualMemory(e) => ("virtual memory error", Some(e)),
            Error::Encoding => ("encoding error", None),
        }
    }

    /// Returns a simple string representation of the error.
    pub fn to_str(self) -> &'static str {
        self.to_str_pair().0
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let (desc, value) = self.to_str_pair();

        if let Some(value) = value {
            write!(f, "{}: {}", desc, value)
        } else {
            f.write_str(desc)
        }
    }
}

#[cfg(feature = "std")]
impl error::Error for Error {
    fn description(&self) -> &str {
        self.to_str()
    }
}

/// Specialized `PartialError` type for recoverable memflow errors.
#[derive(Copy, Clone, Eq, PartialEq, Hash)]
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
    /// Returns a tuple representing the error description and its string value.
    pub fn to_str_pair(&self) -> (&'static str, Option<&'static str>) {
        match self {
            PartialError::Error(e) => ("other error", Some(e.to_str_pair().0)),
            PartialError::PartialVirtualRead(_) => ("partial virtual read error", None),
            PartialError::PartialVirtualWrite => ("partial virtual write error", None),
        }
    }

    /// Returns a simple string representation of the error.
    pub fn to_str(&self) -> &'static str {
        self.to_str_pair().0
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
        let (desc, value) = self.to_str_pair();

        if let Some(value) = value {
            write!(f, "{}: {}", desc, value)
        } else {
            f.write_str(desc)
        }
    }
}

#[cfg(feature = "std")]
impl<T: fmt::Debug> error::Error for PartialError<T> {
    fn description(&self) -> &str {
        self.to_str()
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
            Err(_) => Err(Error::Partial),
        }
    }

    fn data_part(self) -> Result<T> {
        match self {
            Ok(data) => Ok(data),
            Err(PartialError::PartialVirtualRead(data)) => Ok(data),
            //Err(Error::PartialVirtualWrite(data)) => Ok(data),
            Err(_) => Err(Error::Partial),
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
