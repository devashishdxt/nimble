use thiserror::Error;

#[derive(Debug, Error)]
/// Error returned by this crate
pub enum Error {
    /// IO error
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    /// Invalid character
    #[error("Invalid character: {0}")]
    InvalidChar(u32),
    /// Invalid enum variant
    #[error("Invalid enum variant: {0}")]
    InvalidEnumVariant(u32),
    /// Invalid length
    #[error("Invalid length: {0}")]
    InvalidLength(u64),
    /// Invalid UTF-8 string
    #[error("Invalid UTF-8 string: {0}")]
    InvalidUtf8String(#[from] std::string::FromUtf8Error),
    /// Partially filled array
    #[error("Partially filled array")]
    PartiallyFilledArray,
    /// CString contains trailing 0 byte
    #[error("CString contains trailing 0 byte: {0}")]
    NulError(#[from] std::ffi::NulError),
}

/// Result type with [`nimble::Error`](enum.Error.html)
pub type Result<T> = core::result::Result<T, Error>;
