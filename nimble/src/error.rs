use core::fmt;
use std::string::FromUtf8Error;

#[derive(Debug)]
/// Error returned by this crate
pub enum Error {
    /// IO error
    IoError(std::io::Error),
    /// Invalid character
    InvalidChar(u32),
    /// Invalid enum variant
    InvalidEnumVariant(u32),
    /// Invalid length
    InvalidLength(u64),
    /// Invalid UTF-8 string
    InvalidUtf8String(FromUtf8Error),
    /// Partially filled array
    PartiallyFilledArray,
}

impl From<std::io::Error> for Error {
    #[inline]
    fn from(io_error: std::io::Error) -> Self {
        Self::IoError(io_error)
    }
}

impl From<FromUtf8Error> for Error {
    #[inline]
    fn from(error: FromUtf8Error) -> Self {
        Self::InvalidUtf8String(error)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::IoError(ref io_error) => write!(f, "IO Error: {}", io_error),
            Self::InvalidChar(ref code) => write!(f, "Invalid character: {}", code),
            Self::InvalidEnumVariant(ref code) => write!(f, "Invalid enum variant: {}", code),
            Self::InvalidLength(ref len) => write!(f, "Invalid length: {}", len),
            Self::InvalidUtf8String(ref error) => write!(f, "Invalid UTF-8 string: {}", error),
            Self::PartiallyFilledArray => write!(f, "Partially filled array"),
        }
    }
}

impl std::error::Error for Error {}

/// Result type with [`nimble::Error`](enum.Error.html)
pub type Result<T> = core::result::Result<T, Error>;
