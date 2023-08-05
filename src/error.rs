//! Implements the crate's error type

use std::{
    backtrace::{Backtrace, BacktraceStatus},
    error,
    fmt::{self, Display, Formatter},
    num::TryFromIntError,
    str::Utf8Error,
    string::FromUtf8Error,
};

/// Creates a new error
#[macro_export]
macro_rules! error {
    (with: $error:expr, $($arg:tt)*) => {{
        let error = format!($($arg)*);
        let source = Box::new($error);
        $crate::error::Error::new(error, Some(source))
    }};
    ($($arg:tt)*) => {{
        let error = format!($($arg)*);
        $crate::error::Error::new(error, None)
    }};
}

/// The crates error type
#[derive(Debug)]
pub struct Error {
    /// The error description
    pub error: String,
    /// The underlying error
    pub source: Option<Box<dyn error::Error + Send>>,
    /// The backtrace
    pub backtrace: Backtrace,
}
impl Error {
    /// Creates a new error
    #[doc(hidden)]
    pub fn new(error: String, source: Option<Box<dyn error::Error + Send>>) -> Self {
        let backtrace = Backtrace::capture();
        Self { error, source, backtrace }
    }

    /// Whether the error has captured a backtrace or not
    pub fn has_backtrace(&self) -> bool {
        self.backtrace.status() == BacktraceStatus::Captured
    }
}
impl std::error::Error for Error {
    // No members to implement
}
impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        // Print the error
        writeln!(f, "{}", self.error)?;

        // Print the source
        if let Some(source) = &self.source {
            writeln!(f, " caused by: {source}")?;
        }
        Ok(())
    }
}
impl From<FromUtf8Error> for Error {
    fn from(source: FromUtf8Error) -> Self {
        error!(with: source, "UTF-8 decoding error")
    }
}
impl From<Utf8Error> for Error {
    fn from(source: Utf8Error) -> Self {
        error!(with: source, "UTF-8 decoding error")
    }
}
impl From<TryFromIntError> for Error {
    fn from(source: TryFromIntError) -> Self {
        error!(with: source, "integer conversion error")
    }
}
impl From<std::io::Error> for Error {
    fn from(source: std::io::Error) -> Self {
        error!(with: source, "I/O error")
    }
}
impl From<toml::de::Error> for Error {
    fn from(source: toml::de::Error) -> Self {
        error!(with: source, "TOML decoding error")
    }
}
impl From<ehttpd::error::Error> for Error {
    fn from(source: ehttpd::error::Error) -> Self {
        error!(with: source, "ehttpd decoding error")
    }
}
