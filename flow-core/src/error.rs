use std::{convert, error, fmt, result, str};

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
    /// Returns a simple string representation of the error.
    pub fn to_str(self) -> &'static str {
        match self {
            Error::Other(_) => "other error",
            Error::Bounds => "out of bounds",
            Error::InvalidArchitecture => "invalid architecture",
            Error::Connector(_) => "connector error",
            Error::PhysicalMemory(_) => "physical memory error",
            Error::VirtualTranslate => "virtual address translation failed",
            Error::VirtualMemory(_) => "virtual memory error",
            Error::Encoding => "encoding error",
        }
    }

    /// Returns the full string representation of the error.
    pub fn to_string(self) -> String {
        match self {
            Error::Other(e) => format!("other error: {}", e),
            Error::Connector(e) => format!("connector error: {}", e),
            Error::PhysicalMemory(e) => format!("physical write error: {}", e),
            Error::VirtualMemory(e) => format!("virtual write error: {}", e),
            _ => self.to_str().to_string(),
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(&self.to_string())
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        self.to_str()
    }
}

/// Specialized `Result` type for flow-win32 errors.
pub type Result<T> = result::Result<T, Error>;
