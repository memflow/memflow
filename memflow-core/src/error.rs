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
    /// VirtualTranslate error.
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
    fn from(_err: str::Utf8Error) -> Error {
        Error::Encoding
    }
}

impl Error {
    /// Returns a tuple representing the error description and its string value.
    pub fn to_str_pair(self) -> (&'static str, Option<&'static str>) {
        match self {
            Error::Other(e) => ("other error", Some(e)),
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

/// Specialized `Result` type for memflow errors.
pub type Result<T> = result::Result<T, Error>;
