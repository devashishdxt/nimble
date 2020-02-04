#[cfg(feature = "async-std")]
use async_std::io::{Read, ReadExt};

#[cfg(feature = "tokio")]
use tokio::io::{AsyncRead as Read, AsyncReadExt};

use core::{convert::TryFrom, future::Future, pin::Pin};

use crate::error::{Error, Result};

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
                    async fn __decode_from<I>(mut reader: I) -> Result<$type>
                    where
                        I: Read + Unpin + Send,
                    {
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

impl Decode for bool {
    fn decode_from<'t, R>(reader: R) -> Pin<Box<dyn Future<Output = Result<Self>> + Send + 't>>
    where
        R: Read + Unpin + Send + 't,
        Self: 't,
    {
        async fn __decode_from<I>(mut reader: I) -> Result<bool>
        where
            I: Read + Unpin + Send,
        {
            let mut bytes = [0u8; core::mem::size_of::<u8>()];
            reader.read_exact(&mut bytes).await?;
            Ok(<u8>::from_be_bytes(bytes) != 0)
        }

        Box::pin(__decode_from(reader))
    }
}

impl Decode for char {
    fn decode_from<'t, R>(reader: R) -> Pin<Box<dyn Future<Output = Result<Self>> + Send + 't>>
    where
        R: Read + Unpin + Send + 't,
        Self: 't,
    {
        async fn __decode_from<I>(mut reader: I) -> Result<char>
        where
            I: Read + Unpin + Send,
        {
            let mut bytes = [0u8; core::mem::size_of::<u32>()];
            reader.read_exact(&mut bytes).await?;

            let code = <u32>::from_be_bytes(bytes);

            core::char::from_u32(code).ok_or_else(|| Error::InvalidChar(code))
        }

        Box::pin(__decode_from(reader))
    }
}

impl<T> Decode for Option<T>
where
    T: Decode,
{
    fn decode_from<'t, R>(reader: R) -> Pin<Box<dyn Future<Output = Result<Self>> + Send + 't>>
    where
        R: Read + Unpin + Send + 't,
        Self: 't,
    {
        async fn __decode_from<T, I>(mut reader: I) -> Result<Option<T>>
        where
            T: Decode,
            I: Read + Unpin + Send,
        {
            let option = u8::decode_from(&mut reader).await?;

            match option {
                0 => Ok(None),
                1 => T::decode_from(&mut reader).await.map(Some),
                _ => Err(Error::InvalidEnumVariant(option as u32)),
            }
        }

        Box::pin(__decode_from(reader))
    }
}

impl<T> Decode for Vec<T>
where
    T: Decode + Send,
{
    fn decode_from<'t, R>(reader: R) -> Pin<Box<dyn Future<Output = Result<Self>> + Send + 't>>
    where
        R: Read + Unpin + Send + 't,
        Self: 't,
    {
        async fn __decode_from<T, I>(mut reader: I) -> Result<Vec<T>>
        where
            T: Decode + Send,
            I: Read + Unpin + Send,
        {
            let len = u64::decode_from(&mut reader).await?;
            let len = usize::try_from(len).map_err(|_| Error::InvalidVecLength(len))?;

            let mut value = Vec::with_capacity(len);

            for _ in 0..len {
                value.push(T::decode_from(&mut reader).await?);
            }

            Ok(value)
        }

        Box::pin(__decode_from(reader))
    }
}
