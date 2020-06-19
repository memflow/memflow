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
    Initialization(&'static str),
    SymbolStore(&'static str),
    ProcessInfo,
    ModuleInfo,
    /// flow-core error.
    ///
    /// Catch-all for flow-core related errors.
    Core(flow_core::Error),
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
impl From<flow_core::Error> for Error {
    fn from(error: flow_core::Error) -> Error {
        Error::Core(error)
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
    /// Returns a simple string representation of the error.
    pub fn to_str(self) -> &'static str {
        match self {
            Error::Other(_) => "other error",
            Error::Bounds => "out of bounds",
            Error::InvalidArchitecture => "invalid architecture",
            Error::Initialization(_) => "error during initialization",
            Error::SymbolStore(_) => "error in symbol store",
            Error::ProcessInfo => "error retrieving process info",
            Error::ModuleInfo => "error retrieving module info",
            Error::Core(_) => "error in core",
            Error::PDB(_) => "error handling pdb",
            Error::PE(_) => "error handling pe",
            Error::Encoding => "encoding error",
            Error::Unicode(_) => "error reading unicode string",
        }
    }

    /// Returns the full string representation of the error.
    pub fn to_string(self) -> String {
        match self {
            Error::Other(e) => format!("other error: {}", e),
            Error::Initialization(e) => format!("error during initialization: {}", e),
            Error::SymbolStore(e) => format!("error in symbol store: {}", e),
            Error::Core(e) => format!("error in core: {}", e.to_string()),
            Error::PDB(e) => format!("error handling pdb: {}", e),
            Error::PE(e) => format!("error handling pe: {}", e.to_str()),
            Error::Unicode(e) => format!("error reading unicode string: {}", e),
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
