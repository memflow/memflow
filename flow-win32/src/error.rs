pub type Result<T> = std::result::Result<T, Error>;

pub struct Error {
    error: Box<dyn std::error::Error + Send + Sync>,
}

impl Error {
    pub fn new<E>(error: E) -> Error
    where
        E: Into<Box<dyn std::error::Error + Send + Sync>>,
    {
        Error {
            error: error.into(),
        }
    }
}

impl std::fmt::Debug for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(&self.error, f)
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.error)
    }
}

// This is important for other errors to wrap this one.
impl std::error::Error for Error {
    #[allow(deprecated)]
    fn description(&self) -> &str {
        self.error.description()
    }

    #[allow(deprecated)]
    fn cause(&self) -> Option<&dyn std::error::Error> {
        self.error.cause()
    }

    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        // Generic error, underlying cause isn't tracked.
        self.error.source()
    }
}

impl std::convert::From<&str> for Error {
    fn from(error: &str) -> Self {
        Self::new(error)
    }
}

impl std::convert::From<String> for Error {
    fn from(error: String) -> Self {
        Self::new(error)
    }
}

impl std::convert::From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        Self::new(error)
    }
}

impl std::convert::From<flow_core::Error> for Error {
    fn from(error: flow_core::Error) -> Self {
        Self::new(error)
    }
}

impl std::convert::From<Error> for flow_core::Error {
    fn from(error: Error) -> Self {
        Self::new(error)
    }
}

impl std::convert::From<std::str::Utf8Error> for Error {
    fn from(error: std::str::Utf8Error) -> Self {
        Self::new(error)
    }
}
