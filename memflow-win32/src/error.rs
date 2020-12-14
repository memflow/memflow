use std::prelude::v1::*;

use std::{convert, fmt, result, str};

#[cfg(feature = "std")]
use std::error;

// forward declare partial result extension from core for easier access
pub use memflow::error::PartialResultExt;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum Error {
    /// Generic error type containing a string
    Other(&'static str),
    /// Out of bounds.
    ///
    /// Catch-all for bounds check errors.
    Bounds,
    /// Invalid Architecture error.
    ///
    /// The architecture provided is not a valid argument for the given function.
    InvalidArchitecture,
    Initialization(&'static str),
    SymbolStore(&'static str),
    ProcessInfo,
    ModuleInfo,
    /// memflow core error.
    ///
    /// Catch-all for memflow core related errors.
    Core(memflow::error::Error),
    PDB(&'static str),
    /// PE error.
    ///
    /// Catch-all for pe related errors.
    PE(pelite::Error),
    /// Encoding error.
    ///
    /// Catch-all for string related errors such as lacking a nul terminator.
    Encoding,
    /// Unicode error when reading a string from windows.
    ///
    /// Encapsulates all unicode related reading errors.
    Unicode(&'static str),
}

/// Convert from &str to error
impl convert::From<&'static str> for Error {
    fn from(error: &'static str) -> Self {
        Error::Other(error)
    }
}

/// Convert from flow_core::Error
impl From<memflow::error::Error> for Error {
    fn from(error: memflow::error::Error) -> Error {
        Error::Core(error)
    }
}

/// Convert from flow_core::PartialError
impl<T> From<memflow::error::PartialError<T>> for Error {
    fn from(_error: memflow::error::PartialError<T>) -> Error {
        Error::Core(memflow::error::Error::Partial)
    }
}

/// Convert from pelite::Error
impl From<pelite::Error> for Error {
    fn from(error: pelite::Error) -> Error {
        Error::PE(error)
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
            Error::InvalidArchitecture => ("invalid architecture", None),
            Error::Initialization(e) => ("error during initialization", Some(e)),
            Error::SymbolStore(e) => ("error in symbol store", Some(e)),
            Error::ProcessInfo => ("error retrieving process info", None),
            Error::ModuleInfo => ("error retrieving module info", None),
            Error::Core(e) => e.to_str_pair(),
            Error::PDB(e) => ("error handling pdb", Some(e)),
            Error::PE(e) => ("error handling pe", Some(e.to_str())),
            Error::Encoding => ("encoding error", None),
            Error::Unicode(e) => ("error reading unicode string", Some(e)),
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

/// Specialized `Result` type for memflow_win32 errors.
pub type Result<T> = result::Result<T, Error>;
