use core::fmt;

#[derive(Debug)]
pub enum Error {
    IoError(std::io::Error),
    InvalidChar(u32),
    InvalidEnumVariant(u32),
    InvalidVecLength(u64),
}

impl From<std::io::Error> for Error {
    #[inline]
    fn from(io_error: std::io::Error) -> Self {
        Self::IoError(io_error)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::IoError(ref io_error) => write!(f, "IO Error: {}", io_error),
            Self::InvalidChar(ref code) => write!(f, "Invalid character: {}", code),
            Self::InvalidEnumVariant(ref code) => write!(f, "Invalid enum variant: {}", code),
            Self::InvalidVecLength(ref len) => write!(f, "Invalid Vec length: {}", len),
        }
    }
}

impl std::error::Error for Error {}

pub type Result<T> = core::result::Result<T, Error>;
