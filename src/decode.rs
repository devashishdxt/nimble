#[cfg(feature = "async-std")]
use async_std::{io::Read, prelude::*};

#[cfg(feature = "tokio")]
use tokio::io::{AsyncRead as Read, AsyncReadExt};

use core::{future::Future, pin::Pin};

use crate::error::Result;

pub trait Decode: Sized {
    fn decode_from<'t, R>(reader: R) -> Pin<Box<dyn Future<Output = Result<Self>> + Send + 't>>
    where
        R: Read + Unpin + Send + 't,
        Self: 't;
}

macro_rules! impl_primitive {
    ($($type: ty),+) => {
        $(
            impl Decode for $type {
                fn decode_from<'t, R>(reader: R) -> Pin<Box<dyn Future<Output = Result<Self>> + Send + 't>>
                where
                    R: Read + Unpin + Send + 't,
                    Self: 't,
                {
                    async fn __decode_from<T: Read + Unpin>(mut reader: T) -> Result<$type> {
                        let mut bytes = [0u8; core::mem::size_of::<$type>()];
                        reader.read_exact(&mut bytes).await?;
                        Ok(<$type>::from_be_bytes(bytes))
                    }

                    Box::pin(__decode_from(reader))
                }
            }
        )+
    };
}

impl_primitive!(u8, u16, u32, u64, u128, i8, i16, i32, i64, i128, usize, isize);
