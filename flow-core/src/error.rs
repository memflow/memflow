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

// TODO: wait for try_trait to be stabilized
/*
impl std::convert::From<std::option::NoneError> for Error {
    fn from(error: std::option::NoneError) -> Self {
        Self::new(error)
    }
}
*/

impl std::convert::From<std::ffi::NulError> for Error {
    fn from(error: std::ffi::NulError) -> Self {
        Self::new(error)
    }
}

impl std::convert::From<regex::Error> for Error {
    fn from(error: regex::Error) -> Self {
        Self::new(error)
    }
}

#[cfg(feature = "emulator")]
impl std::convert::From<unicorn::Error> for Error {
    fn from(error: unicorn::Error) -> Self {
        Self::new(format!("unicorn error {:?}", error))
    }
}
