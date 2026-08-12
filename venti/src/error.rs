//! Unified error type for the `venti` core.
//!
//! Every `venti` function that can fail returns [`Result`](crate::Result)
//! (= `Result<T, Error>`). The single [`Error::Message`] variant keeps
//! conversions from `&str` / `String` trivial (`Err("…".into())`), and
//! [`Error`] converts back to `String` for logging/interop.

use std::fmt;

/// The `venti` crate-level error type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    /// A human-readable error message.
    Message(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Message(m) => f.write_str(m),
        }
    }
}

impl std::error::Error for Error {}

impl From<&str> for Error {
    fn from(s: &str) -> Self {
        Error::Message(s.to_string())
    }
}

impl From<String> for Error {
    fn from(s: String) -> Self {
        Error::Message(s)
    }
}

impl From<&String> for Error {
    fn from(s: &String) -> Self {
        Error::Message(s.clone())
    }
}

/// Extract the message for logging / host interop.
impl From<Error> for String {
    fn from(e: Error) -> Self {
        match e {
            Error::Message(m) => m,
        }
    }
}

/// Crate-wide result type.
pub type Result<T> = std::result::Result<T, Error>;
