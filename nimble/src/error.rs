use thiserror::Error;

use crate::VarInt;

#[derive(Debug, Error)]
/// Error returned by this crate
pub enum Error {
    /// Invalid character
    #[error("Invalid character: {0}")]
    InvalidChar(u32),
    /// Invalid enum variant
    #[error("Invalid enum variant: {0}")]
    InvalidEnumVariant(VarInt),
    /// Invalid UTF-8 string
    #[error("Invalid UTF-8 string: {0}")]
    InvalidUtf8String(#[from] std::string::FromUtf8Error),
    /// IO error
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    /// A non-zero value is zero
    #[error("A non-zero value is zero")]
    NonZeroError,
    /// CString contains trailing 0 byte
    #[error("CString contains trailing 0 byte: {0}")]
    NulError(#[from] std::ffi::NulError),
    /// Partially filled array
    #[error("Partially filled array")]
    PartiallyFilledArray,
    /// Failed to do integral type conversion
    #[error("Failed to do integral type conversion: {0}")]
    TryFromIntError(#[from] core::num::TryFromIntError),
}

/// Result type with [`nimble::Error`](enum.Error.html)
pub type Result<T> = core::result::Result<T, Error>;
