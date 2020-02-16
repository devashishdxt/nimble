//! Re-exports IO traits from `tokio`/`async-std` depending on enabled feature.
#[cfg(not(feature = "tokio"))]
pub use futures_util::io::{
    AsyncRead as Read, AsyncReadExt as ReadExt, AsyncWrite as Write, AsyncWriteExt as WriteExt,
};

#[cfg(feature = "tokio")]
pub use tokio::io::{
    AsyncRead as Read, AsyncReadExt as ReadExt, AsyncWrite as Write, AsyncWriteExt as WriteExt,
};