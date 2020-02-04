use core::fmt;
use std::string::FromUtf8Error;

#[derive(Debug)]
pub enum Error {
    IoError(std::io::Error),
    InvalidChar(u32),
    InvalidEnumVariant(u32),
    InvalidLength(u64),
    InvalidUtf8String(FromUtf8Error),
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
            Self::InvalidUtf8String(ref error) => write!(f, "Invalid UTF8 String: {}", error),
        }
    }
}

impl std::error::Error for Error {}

pub type Result<T> = core::result::Result<T, Error>;
