#[cfg(feature = "async-std")]
use async_std::io::{Read, ReadExt};

#[cfg(feature = "tokio")]
use tokio::io::{AsyncRead as Read, AsyncReadExt};

use core::{convert::TryFrom, future::Future, pin::Pin};
use std::{borrow::Cow, rc::Rc, sync::Arc};

use arrayvec::ArrayVec;

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

impl<T, E> Decode for core::result::Result<T, E>
where
    T: Decode,
    E: Decode,
{
    fn decode_from<'t, R>(reader: R) -> Pin<Box<dyn Future<Output = Result<Self>> + Send + 't>>
    where
        R: Read + Unpin + Send + 't,
        Self: 't,
    {
        async fn __decode_from<T, E, I>(mut reader: I) -> Result<core::result::Result<T, E>>
        where
            T: Decode,
            E: Decode,
            I: Read + Unpin + Send,
        {
            let option = u8::decode_from(&mut reader).await?;

            match option {
                0 => T::decode_from(&mut reader).await.map(Ok),
                1 => E::decode_from(&mut reader).await.map(Err),
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
            let len = usize::try_from(len).map_err(|_| Error::InvalidLength(len))?;

            let mut value = Vec::with_capacity(len);

            for _ in 0..len {
                value.push(T::decode_from(&mut reader).await?);
            }

            Ok(value)
        }

        Box::pin(__decode_from(reader))
    }
}

impl Decode for String {
    fn decode_from<'t, R>(reader: R) -> Pin<Box<dyn Future<Output = Result<Self>> + Send + 't>>
    where
        R: Read + Unpin + Send + 't,
        Self: 't,
    {
        async fn __decode_from<I>(reader: I) -> Result<String>
        where
            I: Read + Unpin + Send,
        {
            let bytes = <Vec<u8>>::decode_from(reader).await?;
            String::from_utf8(bytes).map_err(Into::into)
        }

        Box::pin(__decode_from(reader))
    }
}

macro_rules! impl_deref {
    ($type: ty, $func: expr) => {
        impl<T> Decode for $type
        where
            T: Decode + Send,
        {
            fn decode_from<'t, R>(
                reader: R,
            ) -> Pin<Box<dyn Future<Output = Result<Self>> + Send + 't>>
            where
                R: Read + Unpin + Send + 't,
                Self: 't,
            {
                async fn __decode_from<T, I>(reader: I) -> Result<$type>
                where
                    T: Decode,
                    I: Read + Unpin + Send,
                {
                    T::decode_from(reader).await.map($func)
                }

                Box::pin(__decode_from(reader))
            }
        }
    };
}

impl_deref!(Box<T>, Box::new);
impl_deref!(Rc<T>, Rc::new);
impl_deref!(Arc<T>, Arc::new);

impl<'a, T: ?Sized> Decode for Cow<'a, T>
where
    T: ToOwned + Send,
    <T as ToOwned>::Owned: Decode,
{
    fn decode_from<'t, R>(reader: R) -> Pin<Box<dyn Future<Output = Result<Self>> + Send + 't>>
    where
        R: Read + Unpin + Send + 't,
        Self: 't,
    {
        async fn __decode_from<'b, T: ?Sized, I>(reader: I) -> Result<Cow<'b, T>>
        where
            T: ToOwned + 'b,
            <T as ToOwned>::Owned: Decode,
            I: Read + Unpin + Send,
        {
            Ok(Cow::Owned(
                <<T as ToOwned>::Owned>::decode_from(reader).await?,
            ))
        }

        Box::pin(__decode_from(reader))
    }
}

macro_rules! impl_fixed_arr {
    ($($len: expr),+) => {
        $(
            impl<T> Decode for [T; $len]
            where
                T: Decode + Send,
            {
                fn decode_from<'t, R>(reader: R) -> Pin<Box<dyn Future<Output = Result<Self>> + Send + 't>>
                where
                    R: Read + Unpin + Send + 't,
                    Self: 't,
                {
                    async fn __decode_from<T, I>(mut reader: I) -> Result<[T; $len]>
                    where
                        T: Decode + Send,
                        I: Read + Unpin + Send,
                    {
                        let mut arr = ArrayVec::<[T; $len]>::new();

                        for _ in 0..$len {
                            let value = T::decode_from(&mut reader).await?;
                            arr.push(value)
                        }

                        arr.into_inner().map_err(|_| Error::PartiallyFilledArray)
                    }

                    Box::pin(__decode_from(reader))
                }
            }
        )+
    };
}

impl_fixed_arr!(
    1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26,
    27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47, 48, 49, 50,
    51, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63, 64, 128, 256, 512, 1024
);
