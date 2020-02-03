use core::fmt;

#[derive(Debug)]
pub enum Error {
    IoError(std::io::Error),
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
            Self::IoError(io_error) => write!(f, "IO Error: {}", io_error),
        }
    }
}

impl std::error::Error for Error {}

pub type Result<T> = core::result::Result<T, Error>;
