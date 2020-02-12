#[cfg(feature = "async-std")]
pub use async_std::io::{prelude::WriteExt, Read, ReadExt, Write};

#[cfg(feature = "tokio")]
pub use tokio::io::{
    AsyncRead as Read, AsyncReadExt as ReadExt, AsyncWrite as Write, AsyncWriteExt as WriteExt,
};
