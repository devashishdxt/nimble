pub mod decode;
pub mod encode;

cfg_if::cfg_if! {
    if #[cfg(feature = "tokio")] {
        use tokio::io::{AsyncWrite as Write, AsyncWriteExt};
    } else if #[cfg(feature = "async-std")] {
        use async_std::{io::Write, prelude::*};
    }
}

use std::io::Result;

use crate::encode::Encode;

#[inline]
pub async fn encode_to<W, V>(writer: &mut W, value: &V) -> Result<usize>
where
    W: Write + Unpin,
    V: Encode,
{
    writer.write(&value.encode()).await
}
